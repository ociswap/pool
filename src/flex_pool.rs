use scrypto::prelude::*;

use crate::constants::*;
use crate::pool_math::*;
use crate::utils::*;
use flex_pool_hooks::*;
use common::math::DivisibilityRounding;
use oracle::{Oracle, AccumulatedObservation, ObservationInterval};
use common::pools::*;
use common::time::*;

#[blueprint]
#[events(InstantiateEvent, SwapEvent, FlashLoanEvent)]
#[types(u16, AccumulatedObservation)]
mod flex_pool {
    enable_method_auth! {
        roles {
            blueprint => updatable_by: [];
        },
        methods {
            swap                        => PUBLIC;
            add_liquidity               => PUBLIC;
            remove_liquidity            => PUBLIC;
            x_share                     => PUBLIC;
            x_address                   => PUBLIC;
            y_address                   => PUBLIC;
            x_divisibility              => PUBLIC;
            y_divisibility              => PUBLIC;
            lp_address                  => PUBLIC;
            price_sqrt                  => PUBLIC;
            vault_amounts               => PUBLIC;
            input_fee_rate              => PUBLIC;
            fee_protocol_share          => PUBLIC;
            flash_loan_fee_rate         => PUBLIC;
            flash_loan                  => PUBLIC;
            repay_loan                  => PUBLIC;
            flash_loan_address          => PUBLIC;
            liquidity_pool              => PUBLIC;
            registry                    => PUBLIC;
            sync_registry               => PUBLIC;
            next_sync_time              => PUBLIC;
            hook                        => PUBLIC;
            observations_limit          => PUBLIC;
            observation                 => PUBLIC;
            observation_intervals       => PUBLIC;
            observations_stored         => PUBLIC;
            last_observation_index      => PUBLIC;
            set_liquidity_pool_meta     => restrict_to: [blueprint];
            execute_after_instantiate   => restrict_to: [blueprint]; // called within instantiate on the newly created component
        }
    }

    struct FlexPool {
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        y_address: ResourceAddress,
        x_divisibility: u8,
        y_divisibility: u8,
        input_fee_rate: Decimal,
        x_share: Decimal,
        ratio: Decimal,
        x_protocol_fee: Vault,
        y_protocol_fee: Vault,
        fee_protocol_share: Decimal,
        liquidity_pool: Global<TwoResourcePool>,
        lp_manager: ResourceManager,
        flash_manager: ResourceManager,
        flash_loan_fee_rate: Decimal,
        registry: Global<AnyComponent>,
        next_sync_time: u64,
        hooks: HashMap<(PackageAddress, String), Global<AnyComponent>>,
        hook_calls: HookCalls,
        hook_badges: HashMap<ComponentAddress, Vault>,

        oracle: Oracle,
    }

    impl FlexPool {
        /// Instantiates a new `FlexPool` with specified parameters.
        ///
        /// This method sets up a new liquidity pool, which can be either balanced or imbalanced. It ensures that the provided
        /// token addresses are valid and different, and that the initial share distribution is within the acceptable bounds.
        /// It also sets up various fees and hooks for additional functionalities.
        ///
        /// ## Arguments
        /// - `a_address`: ResourceAddress for token A used in this pool.
        /// - `b_address`: ResourceAddress for token B used in this pool.
        /// - `input_fee_rate`: Fee rate applied to swap inputs (between 0 and 1).
        /// - `flash_loan_fee_rate`: Fee rate applied to flash loans.
        /// - `a_share`: Share of token A in the pool value. For a balanced 50/50 pool, this would be 0.5. For an imbalanced 80/20 pool, this would be 0.8 or 0.2 depending on the sorting of the addresses.
        /// - `registry_address`: Address of the registry for protocol fee configuration and collection.
        /// - `hook_badges`: Vector of tuples containing hook components and associated badges for access control.
        /// - `dapp_definition`: Address of the dApp definition component.
        ///
        /// ## Returns
        /// A global reference to the instantiated `FlexPool`.
        ///
        /// ## Panics
        /// - If `a_address` and `b_address` are the same.
        /// - If `a_share` is not within the range ]0, 1[.
        /// - If `input_fee_rate` or `flash_loan_fee_rate` are not within valid ranges.
        /// - If either `a_address` or `b_address` do not point to fungible tokens.
        pub fn instantiate(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            input_fee_rate: Decimal,
            flash_loan_fee_rate: Decimal,
            a_share: Decimal,
            registry_address: ComponentAddress,
            hook_badges: Vec<(ComponentAddress, Bucket)>,
            dapp_definition: ComponentAddress,
        ) -> (Global<FlexPool>, ResourceAddress) {
            // Validity assertions
            assert!(
                dec!(0) < a_share && a_share < dec!(1),
                "The share of token A must be strictly between 0 and 1 (excluding bounds)!"
            );

            let (x_address, y_address) = check_and_sort_addresses(a_address, b_address);

            assert_input_fee_rate_is_valid(input_fee_rate);
            assert_flash_loan_fee_rate_is_valid(flash_loan_fee_rate);

            // Ensure both token addresses point to fungible tokens.
            assert!(
                ResourceManager::from_address(x_address)
                    .resource_type()
                    .is_fungible(),
                "[Instantiate]: Address A should be a fungible token."
            );
            assert!(
                ResourceManager::from_address(y_address)
                    .resource_type()
                    .is_fungible(),
                "[Instantiate]: Address B should be a fungible token."
            );

            // Calculate ratio
            let (ratio, x_share) = if x_address == a_address {
                (a_share / (dec!(1) - a_share), a_share) // the larger the larger share of X in the pool
            } else {
                ((dec!(1) - a_share) / a_share, 1 - a_share)
            };

            // Generate and execute hooks for additional functionalities before instantiation.
            let (hook_calls, mut hook_badges_bucket, hooks) = generate_calls_hooks(hook_badges);
            execute_hooks_before_instantiate(
                &hook_calls.before_instantiate,
                &hook_badges_bucket,
                (BeforeInstantiateState {
                    x_address,
                    y_address,
                    price_sqrt: None,
                    input_fee_rate,
                    flash_loan_fee_rate,
                },),
            );

            // Move hook badges from buckets to vaults to store in the component state.
            let hook_badges_vault: HashMap<ComponentAddress, Vault> = hook_badges_bucket
                .drain()
                .map(|(component_address, bucket)| (component_address, Vault::with_bucket(bucket)))
                .collect();

            // Reserve an address for the new pool and set up LP token management.
            let (address_reservation, pool_address) =
                Runtime::allocate_component_address(FlexPool::blueprint_id());

            let pool_access_rule = rule!(require(global_caller(pool_address)));
            let liquidity_pool = Blueprint::<TwoResourcePool>::instantiate(
                OwnerRole::Fixed(pool_access_rule.clone()),
                pool_access_rule,
                (x_address, y_address),
                None,
            );
            let lp_address = lp_address(&liquidity_pool).expect("Unable to read LP address!");
            let lp_manager = ResourceManager::from_address(lp_address);

            // Set up a resource manager for flash loans.
            let flash_manager =
                ResourceBuilder::new_ruid_non_fungible::<FlashLoan>(OwnerRole::None)
                    .mint_roles(mint_roles!(
                        minter => rule!(require(global_caller(pool_address)));
                        minter_updater => rule!(deny_all);
                    ))
                    .burn_roles(burn_roles!(
                        burner => rule!(require(global_caller(pool_address)));
                        burner_updater => rule!(deny_all);
                    ))
                    .deposit_roles(deposit_roles!(
                        depositor => rule!(deny_all);
                        depositor_updater => rule!(deny_all);
                    ))
                    .create_with_no_initial_supply();

            // Collect hook addresses for metadata.
            let hooks_vec: Vec<ComponentAddress> = hook_badges_vault.keys().cloned().collect();

            let x_divisibility = ResourceManager::from_address(x_address)
                .resource_type()
                .divisibility()
                .unwrap_or_default();
            let y_divisibility = ResourceManager::from_address(y_address)
                .resource_type()
                .divisibility()
                .unwrap_or_default();

            let pool = (Self {
                pool_address,
                x_address,
                y_address,
                x_divisibility,
                y_divisibility,
                input_fee_rate,
                flash_loan_fee_rate,
                x_share,
                ratio,
                x_protocol_fee: Vault::new(x_address),
                y_protocol_fee: Vault::new(y_address),
                fee_protocol_share: Decimal::ZERO,
                liquidity_pool,
                lp_manager,
                flash_manager,
                registry: registry_address.into(),
                next_sync_time: 0,
                hook_calls,
                hook_badges: hook_badges_vault,
                hooks,

                oracle: Oracle::new(),
            })
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                blueprint => rule!(require(global_caller(FlexPool::blueprint_id())));
            ))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "pool_address" => pool_address, locked;
                    "liquidity_pool" => liquidity_pool.address(), locked;
                    "lp_address" => lp_address, locked;
                    "flash_loan_address" => flash_manager.address(), locked;
                    "x_address" => x_address, locked;
                    "y_address" => y_address, locked;
                    "input_fee_rate" => input_fee_rate, locked;
                    "flash_loan_fee_rate" => flash_loan_fee_rate, locked;
                    "registry" => registry_address, locked;
                    "hooks" => hooks_vec.clone(), locked;
                    "dapp_definition" => dapp_definition, locked;
                }
            })
            .globalize();

            // Sets the metadata for the liquidity pool.
            pool.set_liquidity_pool_meta(pool_address, lp_address, dapp_definition);

            // Execute post-instantiation hooks and emit an event for successful instantiation.
            pool.execute_after_instantiate(AfterInstantiateState {
                pool_address,
                x_address,
                y_address,
                price_sqrt: None,
                input_fee_rate,
                flash_loan_fee_rate,
            });

            Runtime::emit_event(InstantiateEvent {
                pool_address,
                lp_address,
                x_address,
                y_address,
                x_share,
                price_sqrt: None,
                input_fee_rate,
                flash_loan_fee_rate,
                registry_address,
                liquidity_pool_address: liquidity_pool.address(),
                hooks: hooks_vec,
                dapp_definition,
            });

            (pool, lp_address)
        }

        /// Instantiates a new Basic Pool with initial liquidity.
        ///
        /// This method creates a new `FlexPool` and then adds initial
        /// liquidity.
        ///
        /// ## Arguments
        /// - `a_bucket`: Bucket containing token A for initial liquidity.
        /// - `b_bucket`: Bucket containing token B for initial liquidity.
        /// - `input_fee_rate`: The fee rate charged on swaps (e.g., 0.03 for 3%).
        /// - `flash_loan_fee_rate`: The fee rate charged on flash loans.
        /// - `a_share`: Share of token A in the pool value. For a balanced 50/50 pool, this would be 0.5. For an imbalanced 80/20 pool, this would be 0.8 or 0.2 depending on the sorting of the addresses.
        /// - `registry_address`: The address of the registry for managing protocol configurations.
        /// - `hook_badges`: A vector of tuples pairing component addresses with badges, controlling access to callable hooks.
        /// - `dapp_definition`: Address of the dApp definition component associated with this pool.
        ///
        /// ## Returns
        /// - A tuple containing:
        ///   - A global reference to the instantiated `FlexPool`.
        ///   - A bucket containing the LP tokens representing the initial liquidity position.
        ///   - An optional bucket containing any remaining unallocated tokens.
        ///
        /// ## Panics
        /// - If the token addresses for A and B are the same.
        /// - If the `a_share` is not within the range ]0, 1[.
        /// - If `input_fee_rate` or `flash_loan_fee_rate` are not within valid ranges.
        /// - If either `a_address` or `b_address` do not point to fungible tokens.
        pub fn instantiate_with_liquidity(
            a_bucket: Bucket,
            b_bucket: Bucket,
            input_fee_rate: Decimal,
            flash_loan_fee_rate: Decimal,
            a_share: Decimal,
            registry_address: ComponentAddress,
            hook_badges: Vec<(ComponentAddress, Bucket)>,
            dapp_definition: ComponentAddress,
        ) -> (Global<FlexPool>, Bucket, Option<Bucket>) {
            let (pool, _) = Self::instantiate(
                a_bucket.resource_address(),
                b_bucket.resource_address(),
                input_fee_rate,
                flash_loan_fee_rate,
                a_share,
                registry_address,
                hook_badges,
                dapp_definition,
            );
            let (lp_token, remainder) = pool.add_liquidity(a_bucket, b_bucket);
            (pool, lp_token, remainder)
        }

        /// Add liquidity to the Pool by providing both tokens.
        ///
        /// # Arguments
        /// * `a_bucket`: Bucket with one of the two tokens of the pool.
        /// * `b_bucket`: Bucket with one of the two tokens of the pool.
        /// Order doesn't matter here, the liquidity pool will handle it.
        ///
        /// # Returns
        /// Returns a tuple consisting of:
        /// * LP tokens you get for providing liquidity.
        /// * Some(Bucket)` if we have a remainder or else None.
        pub fn add_liquidity(
            &mut self,
            a_bucket: Bucket,
            b_bucket: Bucket,
        ) -> (Bucket, Option<Bucket>) {
            self.liquidity_pool.contribute((a_bucket, b_bucket))
        }

        /// Remove liquidity from the liquidity pool, using the LP token, and get your tokens x and y back. The fees generated by the swaps
        ///  are contained in these returned tokens.
        /// It is also possible to remove liquidity directly from the liquidity pool using the redeem method.
        ///
        /// # Arguments
        /// * `lp_token`: Bucket with the LP tokens generated after adding liquidity to this Pool.
        ///
        /// # Returns
        /// Returns a tuple consisting of:
        /// * A Bucket which contains tokens A from the LP tokens
        /// * A Bucket which contains tokens B from the LP tokens.
        pub fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket) {
            self.liquidity_pool.redeem(lp_token)
        }

        /// Executes a token swap within the liquidity pool.
        ///
        /// This method handles the entire lifecycle of a token swap, including pre-swap hooks, fee calculations,
        /// the actual swap logic, and post-swap hooks. It ensures that the pool's state is synchronized with the registry,
        /// fees are correctly applied, and the oracle is updated with the new price.
        ///
        /// # Arguments
        /// * `input_bucket`: A `Bucket` containing one of the two tokens in the pool to be swapped for the other token.
        ///
        /// # Returns
        /// * A `Bucket` containing the other token after the swap.
        pub fn swap(&mut self, mut input_bucket: Bucket) -> Bucket {
            // Synchronize the pool's state with the registry to collect protocol fees and update protocol fee share.
            self.sync_registry();

            // Determine the type of swap and retrieve the current vault amounts.
            let swap_type = self.swap_type(input_bucket.resource_address());
            let (x_vault, y_vault) = self.vault_amounts();
            let active_liquidity = PreciseDecimal::from(
                self.lp_manager
                    .total_supply()
                    .expect("LP token has no total supply, but it should always have one"),
            );

            // Initialize the state for BeforeSwap hooks.
            let mut before_swap_state: BeforeSwapState = BeforeSwapState {
                pool_address: self.pool_address,
                swap_type,
                price_sqrt: price_sqrt(x_vault, y_vault, self.ratio).expect("Invalid price"),
                active_liquidity,
                input_fee_rate: self.input_fee_rate,
                fee_protocol_share: self.fee_protocol_share,
            };

            // Execute BeforeSwap hooks, validate the output, and adjust the fee rate.
            let (input_address, input_gross_amount) =
                (input_bucket.resource_address(), input_bucket.amount());
            (before_swap_state, input_bucket) =
                self.execute_hooks(HookCall::BeforeSwap, (before_swap_state, input_bucket));
            assert_hooks_bucket_output_and_address(
                input_gross_amount,
                input_address,
                &input_bucket,
                "BeforeSwap",
            );
            self.set_input_fee_rate(before_swap_state.input_fee_rate);

            // Retrieve the current vault amounts and ensure they are valid.
            let input_amount = input_bucket.amount();
            let mut vault_amounts = self.liquidity_pool.get_vault_amounts();
            let input_vault_amount = vault_amounts
                .remove(&input_address)
                .expect("No input resource!");
            assert!(
                input_vault_amount > Decimal::ZERO,
                "Input reserves are empty!"
            );
            let (&output_address, &output_vault_amount) =
                vault_amounts.first().expect("No output resource");
            assert!(
                output_vault_amount > Decimal::ZERO,
                "Output reserves are empty!"
            );

            // Calculate the net input amount and fees.
            let input_divisibility = self.input_divisibility(swap_type);
            let (input_amount_net, input_fee_lp, input_fee_protocol) = input_amount_net(
                input_amount,
                self.input_fee_rate,
                self.fee_protocol_share,
                input_divisibility,
            );

            // Deposit protocol fees.
            self.deposit_protocol_fees(input_bucket.take(input_fee_protocol));

            // Calculate the output amount based on the swap.
            let output_divisibility = self.output_divisibility(swap_type);
            let output_amount = output_amount(
                input_vault_amount,
                output_vault_amount,
                input_amount_net,
                self.ratio,
                swap_type,
                output_divisibility,
            );

            // Withdraw the output amount and deposit the input bucket.
            let mut output_bucket = self.withdraw(output_address, output_amount);
            self.deposit(input_bucket);

            // Initialize the state for AfterSwap hooks.
            let (x_vault, y_vault) = self.vault_amounts();
            let mut after_swap_state: AfterSwapState = AfterSwapState {
                pool_address: self.pool_address,
                swap_type,
                price_sqrt: price_sqrt(x_vault, y_vault, self.ratio).expect("Invalid price"),
                active_liquidity,
                input_fee_rate: self.input_fee_rate,
                fee_protocol_share: self.fee_protocol_share,
                input_address,
                input_amount: input_amount_net,
                output_address,
                output_amount: output_bucket.amount(),
                input_fee_lp,
                input_fee_protocol,
            };

            // Execute AfterSwap hooks, validate the output, and adjust the fee rate.
            (after_swap_state, output_bucket) =
                self.execute_hooks(HookCall::AfterSwap, (after_swap_state, output_bucket));
            assert_hooks_bucket_output_and_address(
                output_amount,
                output_address,
                &output_bucket,
                "AfterSwap",
            );
            self.set_input_fee_rate(after_swap_state.input_fee_rate);

            // Update the oracle with the new price square root.
            self.oracle.observe(after_swap_state.price_sqrt);

            // Emit a SwapEvent to log the swap details.
            Runtime::emit_event(SwapEvent {
                input_address,
                input_gross_amount,
                input_amount: input_amount_net,
                output_address,
                output_amount,
                output_return_amount: output_bucket.amount(),
                input_fee_lp,
                input_fee_protocol,
                price_sqrt: after_swap_state.price_sqrt,
            });

            output_bucket
        }

        /// Retrieve the resource address of token X in the pool.
        ///
        /// # Returns
        /// * The resource address of token X.
        pub fn x_address(&self) -> ResourceAddress {
            self.x_address
        }

        /// Retrieve the resource address of token Y in the pool.
        ///
        /// # Returns
        /// * The resource address of token Y.
        pub fn y_address(&self) -> ResourceAddress {
            self.y_address
        }

        /// Retrieve the divisibility of the X token in this pool
        ///
        /// # Returns
        /// * The divisibility of the X token
        pub fn x_divisibility(&self) -> u8 {
            self.x_divisibility
        }

        /// Retrieve the divisibility of the Y token in this pool
        ///
        /// # Returns
        /// * The divisibility of the Y token
        pub fn y_divisibility(&self) -> u8 {
            self.y_divisibility
        }

        /// Retrieves the divisibility of the input token based on the swap type.
        ///
        /// # Arguments
        /// * `swap_type` - The type of the swap (BuyX or SellX).
        ///
        /// # Returns
        /// * `u8` - The divisibility of the input token.
        fn input_divisibility(&self, swap_type: SwapType) -> u8 {
            match swap_type {
                SwapType::BuyX => self.y_divisibility(),
                SwapType::SellX => self.x_divisibility(),
            }
        }

        /// Retrieves the divisibility of the output token based on the swap type.
        ///
        /// # Arguments
        /// * `swap_type` - The type of the swap (BuyX or SellX).
        ///
        /// # Returns
        /// * `u8` - The divisibility of the output token.
        fn output_divisibility(&self, swap_type: SwapType) -> u8 {
            match swap_type {
                SwapType::BuyX => self.x_divisibility(),
                SwapType::SellX => self.y_divisibility(),
            }
        }

        /// Retrieve the amounts of tokens X and Y in the pool.
        ///
        /// # Returns
        /// * A tuple with shape (Decimal, Decimal) containing the amounts of token X and token Y.
        pub fn vault_amounts(&self) -> (Decimal, Decimal) {
            let reserves = self.liquidity_pool.get_vault_amounts();

            let x_amount = *reserves
                .get(&self.x_address)
                .expect("Resource does not belong to the pool!");
            let y_amount = *reserves
                .get(&self.y_address)
                .expect("Resource does not belong to the pool!");
            (x_amount, y_amount)
        }

        /// Calculate the square root of the price ratio between token X and token Y.
        ///
        /// # Returns
        /// * An `Option<PreciseDecimal>` representing the square root of the price ratio.
        pub fn price_sqrt(&self) -> Option<PreciseDecimal> {
            let (x_vault, y_vault) = self.vault_amounts();
            price_sqrt(x_vault, y_vault, self.ratio)
        }

        /// Retrieve the resource address of the LP token used in this pool
        ///
        /// # Returns
        /// * The resource address of the LP token NFTs used in this pool
        pub fn lp_address(&self) -> ResourceAddress {
            self.lp_manager.address()
        }

        /// Retrieve the share of token X in the pool's total value.
        ///
        /// # Returns
        /// * A `Decimal` representing the share of token X in the pool's total value.
        pub fn x_share(&self) -> Decimal {
            self.x_share
        }

        /// Retrieves the current input fee rate of the pool
        ///
        /// # Returns
        /// * The current input fee rate of the pool
        pub fn input_fee_rate(&self) -> Decimal {
            self.input_fee_rate
        }

        /// Retrieve the protocol's share of the fees in the pool.
        ///
        /// # Returns
        /// * The protocol's share of the fees as a `Decimal`.
        pub fn fee_protocol_share(&self) -> Decimal {
            self.fee_protocol_share
        }

        /// Retrieve the flash loan fee rate of the pool.
        ///
        /// # Returns
        /// * The flash loan fee rate as a `Decimal`.
        pub fn flash_loan_fee_rate(&self) -> Decimal {
            self.flash_loan_fee_rate
        }

        /// Retrieve the global liquidity pool associated with this pool.
        ///
        /// # Returns
        /// * A `Global<TwoResourcePool>` representing the liquidity pool.
        pub fn liquidity_pool(&self) -> Global<TwoResourcePool> {
            self.liquidity_pool
        }

        /// Retrieves the global registry component associated with this pool.
        /// This registry is crucial as it configures and collects protocol fees,
        /// which are essential for the decentralized management and operational sustainability of the pool.
        ///
        /// # Returns
        /// * `Global<AnyComponent>` - A global reference to the registry component used by this pool.
        pub fn registry(&self) -> Global<AnyComponent> {
            self.registry
        }

        /// Sets the metadata for the LP tokens from the liquidity pool to be displayed in the Wallet.
        /// This method can only be called by the Blueprint.
        ///
        /// # Arguments
        /// * `pool_address`: The address of the pool.
        /// * `lp_address`: The address of the LP tokens.
        /// * `dapp_definition`: The dapp definition of the project.
        pub fn set_liquidity_pool_meta(
            &self,
            pool_address: ComponentAddress,
            lp_address: ResourceAddress,
            dapp_definition: ComponentAddress,
        ) {
            let lp_manager = ResourceManager::from_address(lp_address);

            let x_symbol = token_symbol(self.x_address);
            let y_symbol = token_symbol(self.y_address);
            let (name, description) =
                match x_symbol.zip(y_symbol).map(|(x, y)| format!("{}/{}", x, y)) {
                    Some(pair_symbol) => (
                        format!("Ociswap LP {}", pair_symbol).to_owned(),
                        format!("Ociswap LP token for the {} Flex Pool", pair_symbol).to_owned(),
                    ),
                    None => (
                        "Ociswap LP".to_owned(),
                        "Ociswap LP token for the Flex Pool".to_owned(),
                    ),
                };
            lp_manager.set_metadata("name", name);
            lp_manager.set_metadata("description", description);

            let tags = vec![
                "ociswap".to_owned(),
                "liquidity-pool".to_owned(),
                "lp".to_owned(),
                "dex".to_owned(),
                "defi".to_owned(),
            ];
            lp_manager.set_metadata("tags", tags.to_owned());
            lp_manager.set_metadata(
                "icon_url",
                Url::of("https://ociswap.com/icons/lp_token.png".to_owned()),
            );
            lp_manager.set_metadata(
                "info_url",
                Url::of(
                    format!(
                        "https://ociswap.com/pools/{}",
                        Runtime::bech32_encode_address(pool_address)
                    )
                    .to_owned(),
                ),
            );

            let dapp_definition_global: GlobalAddress = dapp_definition.into();
            lp_manager.set_metadata("dapp_definition", dapp_definition_global);
            lp_manager.lock_updatable_metadata();

            self.liquidity_pool
                .set_metadata("dapp_definition", dapp_definition_global);
        }

        /// Initiates a flash loan for one of the tokens (X or Y).
        /// The loan must be repaid within the same transaction for it to be successful.
        ///
        /// # Arguments
        /// * `address`: The address of the token to be loaned.
        /// * `loan_amount`: The amount of tokens to be loaned.
        ///
        /// # Returns
        /// A tuple containing:
        /// * `Bucket` with the loaned tokens.
        /// * `NonFungibleBucket` with a transient token to ensure the loan is repaid within the same transaction.
        pub fn flash_loan(
            &mut self,
            address: ResourceAddress,
            loan_amount: Decimal,
        ) -> (Bucket, Bucket) {
            let divisibility = ResourceManager::from_address(address)
                .resource_type()
                .divisibility()
                .unwrap();
            let loan_amount = loan_amount.floor_to(divisibility);

            // Calculate the loan fee and add it to the borrowed amount to determine the total amount due.
            let fee = (PreciseDecimal::from(loan_amount) * self.flash_loan_fee_rate)
                .ceil_to(divisibility);

            let flash_loan = FlashLoan {
                address,
                due_amount: loan_amount + fee,
                fee,
            };

            Runtime::emit_event(FlashLoanEvent {
                address: flash_loan.address,
                due_amount: flash_loan.due_amount,
                fee: flash_loan.fee,
            });

            // Mint a transient NFT that encapsulates the terms of the loan for repayment validation.
            let loan_terms = self.flash_manager.mint_ruid_non_fungible(flash_loan);

            (self.withdraw(address, loan_amount), loan_terms)
        }

        /// Repays the loan taken through the flash loan.
        ///
        /// # Arguments
        /// * `loan_repayment`: `Bucket` with the tokens to return.
        /// * `loan_terms`: `NonFungibleBucket` with the transient token to ensure the loan is repaid.
        ///
        /// # Returns
        /// The remainder of the `Bucket` used to repay the loan.
        pub fn repay_loan(
            &mut self,
            mut loan_repayment: Bucket,
            loan_terms: NonFungibleBucket,
        ) -> Bucket {
            assert!(
                loan_terms.resource_address() == self.flash_manager.address(),
                "Incorrect resource passed in for loan terms"
            );

            let transient = loan_terms.non_fungible::<FlashLoan>();
            let terms: FlashLoan = transient.data();

            assert!(
                terms.address == loan_repayment.as_fungible().resource_address(),
                "Incorrect resource to repay loan"
            );

            assert!(
                loan_repayment.amount() >= terms.due_amount,
                "Insufficient repayment given for your loan!"
            );

            // Separate the fee from the repayment amount and deposit it as protocol fees.
            self.deposit_protocol_fees(loan_repayment.take(terms.fee));

            // Calculate the principal amount to be returned to the appropriate vault.
            let loan_amount = terms.due_amount - terms.fee;

            // Return the principal amount to the correct vault based on the token address.
            self.deposit(loan_repayment.take(loan_amount));

            // Burn the loan terms NFT to officially close the loan.
            self.flash_manager.burn(loan_terms);

            loan_repayment
        }

        /// Retrieves the resource address of the transient token used within flash loans.
        ///
        /// # Returns
        /// * `ResourceAddress` - The address of the transient token used in flash loans.
        pub fn flash_loan_address(&self) -> ResourceAddress {
            self.flash_manager.address()
        }

        /// Returns the next scheduled synchronization time with the registry.
        ///
        /// This method provides the timestamp (in seconds since the Unix epoch) when the pool is next set to synchronize its state with the registry.
        ///
        /// # Returns
        /// * `u64` - The Unix timestamp indicating when the next synchronization with the registry is scheduled.
        pub fn next_sync_time(&self) -> u64 {
            self.next_sync_time
        }

        /// Synchronizes the pool's state with the registry to potentially update the protocol fees.
        ///
        /// This method is crucial for maintaining the pool's alignment with the broader protocol's fee structure,
        /// which may change over time due to governance actions or other external factors. It ensures that the pool
        /// operates with the most current fee settings, which is essential for correct fee distribution and protocol sustainability.
        ///
        /// If the current time is less than `next_sync_time`, the function exits early to throttle the frequency of updates,
        /// which helps in reducing unnecessary computations and state changes.
        pub fn sync_registry(&mut self) {
            if Clock::time_in_seconds() < self.next_sync_time {
                return;
            }

            // Calls the `sync` method on the registry component, passing the current pool address and the total protocol fees collected since the last sync.
            let (fee_protocol_share, next_sync_time) =
                self.registry
                    .call::<(ComponentAddress, Bucket, Bucket), (Decimal, u64)>(
                        "sync",
                        &(
                            self.pool_address,
                            self.x_protocol_fee.take_all(),
                            self.y_protocol_fee.take_all(),
                        ),
                    );

            // Updates the pool's state with the new protocol fee share and the next allowed sync time.
            self.set_fee_protocol_share(fee_protocol_share);
            self.next_sync_time = next_sync_time;
        }

        /// Sets the input fee rate for the pool.
        ///
        /// Updates the pool's `input_fee_rate` after validating it, ensuring correct fee calculations for transactions.
        ///
        /// # Arguments
        /// * `input_fee_rate` - A `Decimal` representing the new fee rate to be applied.
        ///                      The valid range for this rate is between zero and one, where a value of `0.003` equates to a fee rate of 3%.
        ///
        /// # Panics
        /// Panics if the `input_fee_rate` is not valid as determined by `assert_input_fee_rate_is_valid`.
        fn set_input_fee_rate(&mut self, input_fee_rate: Decimal) {
            assert_input_fee_rate_is_valid(input_fee_rate);
            self.input_fee_rate = input_fee_rate;
        }

        /// Sets the protocol fee share for the pool.
        ///
        /// This method updates the `fee_protocol_share` state of the pool. It ensures that the value is within the allowed range [0, `FEE_PROTOCOL_SHARE_MAX`].
        /// The clamping is crucial to prevent setting a fee share that exceeds the maximum allowed limit, which could lead to incorrect fee calculations.
        ///
        /// # Arguments
        /// * `fee_protocol_share` - A `Decimal` representing the new protocol fee share to be set.
        fn set_fee_protocol_share(&mut self, fee_protocol_share: Decimal) {
            self.fee_protocol_share = fee_protocol_share.clamp(dec!(0), FEE_PROTOCOL_SHARE_MAX);
        }

        /// Executes post-instantiation hooks to extend pool functionality.
        ///
        /// # Arguments
        /// * `after_instantiate_state`: The state information specific to this pool that will be passed to the hooks for processing.
        pub fn execute_after_instantiate(&self, after_instantiate_state: AfterInstantiateState) {
            self.execute_hooks(HookCall::AfterInstantiate, (after_instantiate_state,));
        }

        /// Executes predefined hooks based on the lifecycle event of the pool.
        ///
        /// This method applies custom logic at different stages of the pool's lifecycle,
        /// like before or after swaps, and during liquidity changes or initialization.
        /// It uses hooks to implement modular, event-driven logic that can be customized and linked to these events.
        ///
        /// # Arguments
        /// * `hook_call` - An enum representing the specific lifecycle event.
        /// * `hook_args` - The arguments to pass to the hook functions, allowing for context-specific actions.
        ///
        /// # Returns
        /// Returns the modified hook arguments after all relevant hooks have been executed,
        /// which may carry state changes enacted by the hooks.
        fn execute_hooks<T: ScryptoSbor>(&self, hook_call: HookCall, hook_args: T) -> T {
            let empty = ("".into(), vec![]);
            let hooks = match hook_call {
                HookCall::BeforeInstantiate => &self.hook_calls.before_instantiate,
                HookCall::AfterInstantiate => &self.hook_calls.after_instantiate,
                HookCall::BeforeSwap => &self.hook_calls.before_swap,
                HookCall::AfterSwap => &self.hook_calls.after_swap,
                _ => &empty,
            };
            execute_hooks(&hooks, &self.hook_badges, hook_args)
        }

        /// Retrieves a registered hook component based on its package address and blueprint name.
        ///
        /// This method is crucial for the dynamic invocation of specific functionalities encapsulated within
        /// different components of the system. By providing a package address and a blueprint name, it allows
        /// for the retrieval of the corresponding component if it has been previously registered in the `hooks` map.
        /// This is particularly useful for extending or modifying behavior at runtime without altering the underlying
        /// blueprint code.
        ///
        /// # Arguments
        /// * `package_address` - The address of the package where the component is defined.
        /// * `blueprint_name` - The name of the blueprint within the specified package.
        ///
        /// # Returns
        /// An `Option<Global<AnyComponent>>` which is:
        /// - `Some(Global<AnyComponent>)` if the hook is found, allowing further interaction with the component.
        /// - `None` if no such hook is registered, indicating the absence of the component under the specified identifiers.
        pub fn hook(
            &self,
            package_address: PackageAddress,
            blueprint_name: String,
        ) -> Option<Global<AnyComponent>> {
            self.hooks
                .get(&(package_address, blueprint_name))
                .map(|hook| hook.to_owned())
        }

        // PRIVATE

        /// Withdraws a specified amount of a resource from the liquidity pool.
        ///
        /// # Arguments
        /// * `resource_address` - The address of the resource to withdraw.
        /// * `amount` - The amount of the resource to withdraw.
        ///
        /// # Returns
        /// A `Bucket` containing the withdrawn resource.
        fn withdraw(&mut self, resource_address: ResourceAddress, amount: Decimal) -> Bucket {
            self.liquidity_pool.protected_withdraw(
                resource_address,
                amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            )
        }

        /// Deposits a bucket of resources into the liquidity pool.
        ///
        /// # Arguments
        /// * `bucket` - A `Bucket` containing the resources to deposit.
        fn deposit(&mut self, bucket: Bucket) {
            self.liquidity_pool.protected_deposit(bucket)
        }

        /// Deposits protocol fees into the appropriate protocol fee vault.
        ///
        /// # Arguments
        /// * `bucket` - A mutable `Bucket` containing the resources from which fees will be taken.
        /// * `fee_amount` - The amount of fees to be deposited.
        ///
        /// # Returns
        /// A `Bucket` containing the remaining resources after the fees have been taken.
        fn deposit_protocol_fees(&mut self, fees: Bucket) {
            if fees.resource_address() == self.x_address {
                self.x_protocol_fee.put(fees);
            } else {
                self.y_protocol_fee.put(fees);
            }
        }

        /// Determines the type of swap based on the input resource address.
        ///
        /// # Arguments
        /// * `input_address` - The address of the input resource.
        ///
        /// # Returns
        /// A `SwapType` indicating whether the swap is a sell or buy of resource X.
        fn swap_type(&self, input_address: ResourceAddress) -> SwapType {
            if input_address == self.x_address {
                return SwapType::SellX;
            }
            SwapType::BuyX
        }

        // ORACLE

        /// Fetches an `AccumulatedObservation` for a specified timestamp.
        ///
        /// This method is crucial for providing accurate and timely price data to the pool's trading operations.
        /// It handles different scenarios based on the provided timestamp:
        ///
        /// - **Existing Observation**: Directly returns the observation if it matches the provided timestamp.
        /// - **Interpolation Needed**: If the timestamp falls between two stored observations, it computes an interpolated observation using the closest available data points.
        /// - **Recent Timestamp**: Generates a new observation if the timestamp is more recent than the latest stored but still within the current time bounds.
        /// - **Out of Bounds**: Triggers a panic for timestamps that are out of the valid range, as they cannot be reliably processed.
        pub fn observation(&self, timestamp: u64) -> AccumulatedObservation {
            self.oracle.observation(timestamp)
        }

        /// Calculates the average price square root over specified time intervals.
        ///
        /// This method is essential for determining the geometric mean of the price square root (`price_sqrt`)
        /// between pairs of timestamps. Each pair in the vector represents a start and end timestamp, defining
        /// an interval for which the average `price_sqrt` is computed. This computation is crucial for financial
        /// analyses and operations that require historical price data over specific periods.
        ///
        /// # Arguments
        /// * `intervals` - A vector of tuples where each tuple contains two Unix timestamps (u64). The first element
        ///   is the start timestamp, and the second is the end timestamp of the interval.
        ///
        /// # Returns
        /// A vector of `ObservationInterval` structs, each representing the average `price_sqrt` over the given interval.
        ///
        /// # Example
        /// ```
        /// let intervals = vec![(1609459200, 1609545600), (1609545600, 1609632000)];
        /// let observation_intervals = pool.observation_intervals(intervals);
        /// ```
        pub fn observation_intervals(
            &self,
            intervals: Vec<(u64, u64)>, // In Unix seconds
        ) -> Vec<ObservationInterval> {
            self.oracle.observation_intervals(intervals)
        }

        /// Returns the maximum number of observations that the oracle can store.
        ///
        /// # Returns
        /// A `u16` representing the maximum number of observations that can be stored.
        pub fn observations_limit(&self) -> u16 {
            self.oracle.observations_limit()
        }

        /// Returns the current number of observations stored in the oracle.
        ///
        /// # Returns
        /// A `u16` representing the current number of observations stored.
        pub fn observations_stored(&self) -> u16 {
            self.oracle.observations_stored()
        }

        /// Returns the index of the most recent observation stored in the oracle (for testing).
        ///
        /// # Returns
        /// An `Option<u16>` representing the index of the last observation if it exists, or `None` if no observations have been stored yet.
        pub fn last_observation_index(&self) -> Option<u16> {
            self.oracle.last_observation_index()
        }
    }
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct FlashLoan {
    pub address: ResourceAddress,
    pub due_amount: Decimal,
    pub fee: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct InstantiateEvent {
    pool_address: ComponentAddress,
    lp_address: ResourceAddress,
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    x_share: Decimal,
    price_sqrt: Option<PreciseDecimal>,
    input_fee_rate: Decimal,
    flash_loan_fee_rate: Decimal,
    registry_address: ComponentAddress,
    liquidity_pool_address: ComponentAddress,
    hooks: Vec<ComponentAddress>,
    dapp_definition: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct SwapEvent {
    input_address: ResourceAddress,
    input_amount: Decimal,
    input_gross_amount: Decimal,
    input_fee_lp: Decimal,
    input_fee_protocol: Decimal,
    output_address: ResourceAddress,
    output_amount: Decimal,
    output_return_amount: Decimal,
    price_sqrt: PreciseDecimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct FlashLoanEvent {
    address: ResourceAddress,
    due_amount: Decimal,
    fee: Decimal,
}
