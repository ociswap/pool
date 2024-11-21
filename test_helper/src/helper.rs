use std::mem;

use pretty_assertions::assert_eq;
use radix_engine::system::system_modules::execution_trace::ResourceSpecifier::{self, Amount};
use radix_transactions::builder::ManifestBuilder;
use registry_test_helper::RegistryTestHelper;
use scrypto::prelude::*;
use scrypto_testenv::*;

pub struct FlexPoolTestHelper {
    pub pool_address: Option<ComponentAddress>,
    pub lp_address: Option<ResourceAddress>,
    pub liquidity_pool_address: Option<ComponentAddress>,
    pub registry: RegistryTestHelper,
}

impl FlexPoolTestHelper {
    pub fn new() -> FlexPoolTestHelper {
        Self::new_internal(true)
    }

    pub fn new_without_instantiate_registry() -> FlexPoolTestHelper {
        Self::new_internal(false)
    }

    fn new_internal(instantiate_registry: bool) -> FlexPoolTestHelper {
        let packages: HashMap<&str, &str> = vec![("registry", "registry"), ("flex_pool", ".")]
            .into_iter()
            .collect();
        Self::new_with_packages(packages, instantiate_registry)
    }

    pub fn new_with_packages(
        packages: HashMap<&str, &str>,
        instantiate_registry: bool,
    ) -> FlexPoolTestHelper {
        let mut helper = FlexPoolTestHelper {
            pool_address: None,
            lp_address: None,
            liquidity_pool_address: None,
            registry: RegistryTestHelper::new_with_packages(packages),
        };

        if instantiate_registry {
            helper
                .registry
                .instantiate_default(helper.registry.admin_badge_address());
        }
        helper
    }

    pub fn instantiate_full(
        &mut self,
        a_address: ResourceAddress,
        b_address: ResourceAddress,
        input_fee_rate: Decimal,
        flash_loan_fee_rate: Decimal,
        a_share: Decimal,
        registry: ComponentAddress,
    ) -> &mut FlexPoolTestHelper {
        let package_address = self.registry.env.package_address("flex_pool");
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        let hooks_buckets: Vec<(ComponentAddress, ManifestBucket)> = Vec::new();
        self.registry.env.manifest_builder = manifest_builder.call_function(
            package_address,
            "FlexPool",
            "instantiate",
            manifest_args!(
                a_address,
                b_address,
                input_fee_rate,
                flash_loan_fee_rate,
                a_share,
                registry,
                hooks_buckets,
                self.registry.env.dapp_definition
            ),
        );
        self.registry.env.new_instruction("instantiate", 1, 0);
        self
    }

    pub fn instantiate_full_direct(
        &mut self,
        a_address: ResourceAddress,
        b_address: ResourceAddress,
        input_fee_rate: Decimal,
        flash_loan_fee_rate: Decimal,
        a_share: Decimal,
        registry: ComponentAddress,
        verbose: bool,
    ) -> Receipt {
        self.instantiate_full(
            a_address,
            b_address,
            input_fee_rate,
            flash_loan_fee_rate,
            a_share,
            registry,
        );
        let receipt = self.registry.execute_expect_success(verbose);
        let commit_result = receipt.execution_receipt.expect_commit_success();
        let (pool_address, lp_address): (ComponentAddress, ResourceAddress) =
            receipt.outputs("instantiate")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(lp_address);
        self.liquidity_pool_address = Some(commit_result.new_component_addresses()[1]);
        receipt
    }

    pub fn instantiate(
        &mut self,
        a_address: ResourceAddress,
        b_address: ResourceAddress,
        input_fee_rate: Decimal,
        a_share: Decimal,
        registry: ComponentAddress,
    ) -> &mut FlexPoolTestHelper {
        self.instantiate_full(
            a_address,
            b_address,
            input_fee_rate,
            dec!(0.009),
            a_share,
            registry,
        )
    }

    pub fn instantiate_with_liquidity(
        &mut self,
        a_address: ResourceAddress,
        a_amount: Decimal,
        b_address: ResourceAddress,
        b_amount: Decimal,
        input_fee_rate: Decimal,
        registry: ComponentAddress,
    ) -> &mut FlexPoolTestHelper {
        let account = self.registry.env.account;
        let package_address = self.registry.env.package_address("flex_pool");
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        let hooks_buckets: Vec<(ComponentAddress, ManifestBucket)> = Vec::new();
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account, a_address, a_amount)
            .withdraw_from_account(account, b_address, b_amount)
            .take_from_worktop(a_address, a_amount, self.registry.name("a_bucket"))
            .take_from_worktop(b_address, b_amount, self.registry.name("b_bucket"))
            .with_name_lookup(|builder, lookup| {
                let a_bucket = lookup.bucket(self.registry.name("a_bucket"));
                let b_bucket = lookup.bucket(self.registry.name("b_bucket"));
                builder.call_function(
                    package_address,
                    "FlexPool",
                    "instantiate_with_liquidity",
                    manifest_args!(
                        a_bucket,
                        b_bucket,
                        input_fee_rate,
                        dec!(0.009),
                        dec!(0.5),
                        registry,
                        hooks_buckets,
                        self.registry.env.dapp_definition
                    ),
                )
            });
        self.registry
            .env
            .new_instruction("instantiate_with_liquidity", 5, 4);
        self
    }

    pub fn add_liquidity(
        &mut self,
        x_address: ResourceAddress,
        x_amount: Decimal,
        y_address: ResourceAddress,
        y_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let account_component = self.registry.env.account;
        let pool_address = self.pool_address.unwrap();
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, x_address, x_amount)
            .withdraw_from_account(account_component, y_address, y_amount)
            .take_from_worktop(x_address, x_amount, self.registry.name("x_bucket"))
            .take_from_worktop(y_address, y_amount, self.registry.name("y_bucket"))
            .with_name_lookup(|builder, lookup| {
                let x_bucket = lookup.bucket(self.registry.name("x_bucket"));
                let y_bucket = lookup.bucket(self.registry.name("y_bucket"));
                builder.call_method(
                    pool_address,
                    "add_liquidity",
                    manifest_args!(x_bucket, y_bucket),
                )
            });
        self.registry.env.new_instruction("add_liquidity", 5, 4);
        self
    }

    pub fn remove_liquidity(
        &mut self,
        lp_address: ResourceAddress,
        lp_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let account_component = self.registry.env.account;
        let pool_address = self.pool_address.unwrap();
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, lp_address, lp_amount)
            .take_from_worktop(lp_address, lp_amount, self.registry.name("lp_bucket"))
            .with_name_lookup(|builder, lookup| {
                let lp_bucket = lookup.bucket(self.registry.name("lp_bucket"));
                builder.call_method(pool_address, "remove_liquidity", manifest_args!(lp_bucket))
            });
        self.registry.env.new_instruction("remove_liquidity", 3, 2);
        self
    }

    pub fn removable_liquidity(&mut self, lp_amount: Decimal) -> &mut FlexPoolTestHelper {
        let pool_address = self.pool_address.unwrap();
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            pool_address,
            "removable_liquidity",
            manifest_args!(lp_amount),
        );
        self.registry
            .env
            .new_instruction("removable_liquidity", 1, 0);
        self
    }

    pub fn redeem(
        &mut self,
        lp_address: ResourceAddress,
        lp_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let account_component = self.registry.env.account;
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, lp_address, lp_amount)
            .take_from_worktop(lp_address, lp_amount, self.registry.name("lp_bucket"))
            .with_name_lookup(|builder, lookup| {
                let lp_bucket = lookup.bucket(self.registry.name("lp_bucket"));
                builder.call_method(
                    self.liquidity_pool_address.unwrap(),
                    "redeem",
                    manifest_args!(lp_bucket),
                )
            });
        self.registry.env.new_instruction("redeem", 3, 2);
        self
    }

    pub fn contribute(
        &mut self,
        x_address: ResourceAddress,
        x_amount: Decimal,
        y_address: ResourceAddress,
        y_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let account_component = self.registry.env.account;
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, x_address, x_amount)
            .withdraw_from_account(account_component, y_address, y_amount)
            .take_from_worktop(x_address, x_amount, self.registry.name("x_bucket"))
            .take_from_worktop(y_address, y_amount, self.registry.name("y_bucket"))
            .with_name_lookup(|builder, lookup| {
                let x_bucket = lookup.bucket(self.registry.name("x_bucket"));
                let y_bucket = lookup.bucket(self.registry.name("y_bucket"));
                builder.call_method(
                    self.liquidity_pool_address.unwrap(),
                    "contribute",
                    manifest_args!(x_bucket, y_bucket),
                )
            });
        self.registry.env.new_instruction("contribute", 5, 4);
        self
    }

    pub fn protected_deposit(
        &mut self,
        input_address: ResourceAddress,
        input_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(self.registry.env.account, input_address, input_amount)
            .take_from_worktop(
                input_address,
                input_amount,
                self.registry.name("input_bucket"),
            )
            .with_name_lookup(|builder, lookup| {
                let input_bucket = lookup.bucket(self.registry.name("input_bucket"));
                builder.call_method(
                    self.liquidity_pool_address.unwrap(),
                    "protected_deposit",
                    manifest_args!(input_bucket),
                )
            });
        self.registry.env.new_instruction("protected_deposit", 3, 2);
        self
    }

    pub fn protected_withdraw(
        &mut self,
        output_address: ResourceAddress,
        output_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            self.liquidity_pool_address.unwrap(),
            "protected_withdraw",
            manifest_args!(
                output_address,
                output_amount,
                WithdrawStrategy::Rounded(RoundingMode::ToZero)
            ),
        );
        self.registry
            .env
            .new_instruction("protected_withdraw", 1, 0);
        self
    }

    pub fn getter(&mut self, name: &str) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder =
            manifest_builder.call_method(self.pool_address.unwrap(), name, manifest_args!());
        self.registry.env.new_instruction(name, 1, 0);
        self
    }

    pub fn total_liquidity(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("total_liquidity")
    }

    pub fn liquidity(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("liquidity")
    }

    pub fn price_sqrt(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("price_sqrt")
    }

    pub fn x_share(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("x_share")
    }

    pub fn lp_address(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("lp_address")
    }

    pub fn input_fee_rate(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("input_fee_rate")
    }

    pub fn fee_protocol_share(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("fee_protocol_share")
    }

    pub fn flash_loan_fee_rate(&mut self) -> &mut FlexPoolTestHelper {
        self.getter("flash_loan_fee_rate")
    }

    pub fn swap(
        &mut self,
        input_address: ResourceAddress,
        input_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(self.registry.env.account, input_address, input_amount)
            .take_from_worktop(
                input_address,
                input_amount,
                self.registry.name("input_bucket"),
            )
            .with_name_lookup(|builder, lookup| {
                let input_bucket = lookup.bucket(self.registry.name("input_bucket"));
                builder.call_method(
                    self.pool_address.unwrap(),
                    "swap",
                    manifest_args!(input_bucket),
                )
            });
        self.registry.env.new_instruction("swap", 3, 2);
        self
    }

    pub fn instantiate_default(&mut self, verbose: bool) -> Receipt {
        self.instantiate_default_with_input_fee(dec!(0), verbose)
    }

    pub fn instantiate_default_with_input_fee(
        &mut self,
        fee_rate: Decimal,
        verbose: bool,
    ) -> Receipt {
        self.set_whitelist_registry();
        self.instantiate(
            self.x_address(),
            self.y_address(),
            fee_rate,
            dec!(0.5),
            self.registry.registry_address.unwrap(),
        );
        let receipt = self.registry.execute_expect_success(verbose);
        let commit_result = receipt.execution_receipt.expect_commit_success();
        let (pool_address, lp_address): (ComponentAddress, ResourceAddress) =
            receipt.outputs("instantiate")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(lp_address);
        self.liquidity_pool_address = Some(commit_result.new_component_addresses()[1]);
        receipt
    }

    pub fn instantiate_default_with_all_fees(
        &mut self,
        fee_rate: Decimal,
        fee_protocol_share: Decimal,
        a_share: Decimal,
        verbose: bool,
    ) -> Receipt {
        self.registry.instantiate_execute(
            self.registry.admin_badge_address(),
            fee_protocol_share,
            1,
            1,
        );
        self.set_whitelist_registry();
        self.instantiate(
            self.x_address(),
            self.y_address(),
            fee_rate,
            a_share,
            self.registry.registry_address.unwrap(),
        );
        let receipt = self.registry.execute_expect_success(verbose);
        let commit_result = receipt.execution_receipt.expect_commit_success();
        let (pool_address, lp_address): (ComponentAddress, ResourceAddress) =
            receipt.outputs("instantiate")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(lp_address);
        self.liquidity_pool_address = Some(commit_result.new_component_addresses()[1]);
        receipt
    }

    pub fn instantiate_pool_only(&mut self, verbose: bool) -> Receipt {
        self.instantiate(
            self.x_address(),
            self.y_address(),
            dec!(0),
            dec!(0.5),
            self.registry.registry_address.unwrap(),
        );
        let receipt = self.registry.execute_expect_success(verbose);
        let commit_result = receipt.execution_receipt.expect_commit_success();
        let (pool_address, lp_address): (ComponentAddress, ResourceAddress) =
            receipt.outputs("instantiate")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(lp_address);
        self.liquidity_pool_address = Some(commit_result.new_component_addresses()[1]);
        receipt
    }

    pub fn instantiate_pool_with_hooks(
        &mut self,
        a_address: ResourceAddress,
        b_address: ResourceAddress,
        hooks: Vec<(ComponentAddress, ResourceAddress)>,
    ) -> &mut FlexPoolTestHelper {
        let package_address = self.registry.env.package_address("flex_pool");
        let mut manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        for hook in hooks.clone() {
            let (_, badge_address) = hook;
            manifest_builder = manifest_builder
                .withdraw_from_account(self.registry.env.account, badge_address, dec!(1))
                .take_from_worktop(
                    badge_address,
                    dec!(1),
                    self.registry.name(&badge_address.to_hex()),
                );
        }
        self.registry.env.manifest_builder =
            manifest_builder.with_name_lookup(|builder, lookup| {
                let hooks_buckets: Vec<(ComponentAddress, ManifestBucket)> = hooks
                    .iter()
                    .map(|(component_address, badge_address)| {
                        (
                            component_address.clone(),
                            lookup.bucket(self.registry.name(&badge_address.to_hex())),
                        )
                    })
                    .collect();
                builder.call_function(
                    package_address,
                    "FlexPool",
                    "instantiate",
                    manifest_args!(
                        a_address,
                        b_address,
                        dec!(0),
                        dec!(0.009),
                        dec!(0.5),
                        self.registry.registry_address.unwrap(),
                        hooks_buckets,
                        self.registry.env.dapp_definition
                    ),
                )
            });

        self.registry
            .env
            .new_instruction("instantiate", hooks.len() * 2 + 1, hooks.len() * 2);
        self
    }

    pub fn instantiate_default_with_hooks(
        &mut self,
        hooks: Vec<(ComponentAddress, ResourceAddress)>,
        verbose: bool,
    ) -> &mut FlexPoolTestHelper {
        self.instantiate_pool_with_hooks(
            self.registry.x_address(),
            self.registry.y_address(),
            hooks,
        );
        let receipt = self.registry.execute_expect_success(verbose);
        let (pool_address, lp_address): (ComponentAddress, ResourceAddress) =
            receipt.outputs("instantiate")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(lp_address);
        self
    }

    pub fn add_liquidity_default(
        &mut self,
        x_amount: Decimal,
        y_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        self.add_liquidity(self.x_address(), x_amount, self.y_address(), y_amount);
        self
    }

    pub fn add_liquidity_default_execute(
        &mut self,
        x_amount: Decimal,
        y_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let receipt = self
            .add_liquidity(
                self.registry.x_address(),
                x_amount,
                self.registry.y_address(),
                y_amount,
            )
            .registry
            .execute_expect_success(false);
        let output_buckets = receipt.output_buckets("add_liquidity");
        self.lp_address = Some(output_buckets[0][0].address());
        self
    }

    pub fn swap_x_default(&mut self, x_amount: Decimal) -> &mut FlexPoolTestHelper {
        self.swap(self.x_address(), x_amount);
        self
    }

    pub fn swap_y_default(&mut self, y_amount: Decimal) -> &mut FlexPoolTestHelper {
        self.swap(self.y_address(), y_amount);
        self
    }

    pub fn remove_liquidity_default(&mut self, lp_amount: Decimal) -> &mut FlexPoolTestHelper {
        self.remove_liquidity(self.lp_address.unwrap(), lp_amount)
    }

    pub fn flash_loan(
        &mut self,
        loan_address: ResourceAddress,
        loan_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            self.pool_address.unwrap(),
            "flash_loan",
            manifest_args!(loan_address, loan_amount),
        );
        self.registry.env.new_instruction("flash_loan", 1, 0);
        self
    }

    pub fn repay_loan(
        &mut self,
        repay_address: ResourceAddress,
        repay_amount: Decimal,
        repay_fee_amount: Decimal,
        flash_loan_address: ResourceAddress,
        transient_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        let account_component = self.registry.env.account;
        self.registry.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, repay_address, repay_fee_amount)
            .take_from_worktop(
                repay_address,
                repay_amount + repay_fee_amount,
                self.registry.name("repay_bucket"),
            )
            .take_from_worktop(
                flash_loan_address,
                transient_amount,
                self.registry.name("transient_bucket"),
            )
            .with_name_lookup(|builder, lookup| {
                let repay_bucket = lookup.bucket(self.registry.name("repay_bucket"));
                let transient_bucket = lookup.bucket(self.registry.name("transient_bucket"));
                builder.call_method(
                    self.pool_address.unwrap(),
                    "repay_loan",
                    manifest_args!(repay_bucket, transient_bucket),
                )
            });
        self.registry.env.new_instruction("repay_loan", 4, 3);
        self
    }

    pub fn repay_loan_success(
        &mut self,
        repay_address: ResourceAddress,
        repay_amount: Decimal,
        repay_fee_amount: Decimal,
        remainder_expected: Decimal,
    ) -> &mut FlexPoolTestHelper {
        self.instantiate_default(false);
        let receipt = self
            .add_liquidity_default(dec!(10), dec!(10))
            .flash_loan_address()
            .registry
            .execute_expect_success(false);
        let flash_loan_address: ResourceAddress = receipt.outputs("flash_loan_address")[0];

        self.flash_loan(self.x_address(), dec!(1));
        let receipt = self
            .repay_loan(
                repay_address,
                repay_amount,
                repay_fee_amount,
                flash_loan_address,
                dec!(1),
            )
            .registry
            .execute_expect_success(false);
        let output_buckets = receipt.output_buckets("repay_loan");
        assert_eq!(
            output_buckets,
            vec![vec![Amount(repay_address, remainder_expected)]]
        );

        self
    }

    pub fn repay_loan_failure(
        &mut self,
        repay_address: ResourceAddress,
        repay_amount: Decimal,
        repay_fee_amount: Decimal,
    ) -> &mut FlexPoolTestHelper {
        self.instantiate_default(false);
        let receipt = self
            .add_liquidity_default(dec!(10), dec!(10))
            .flash_loan_address()
            .registry
            .execute_expect_success(false);
        let flash_loan_address: ResourceAddress = receipt.outputs("flash_loan_address")[0];

        self.flash_loan(self.x_address(), dec!(1));
        self.repay_loan(
            repay_address,
            repay_amount,
            repay_fee_amount,
            flash_loan_address,
            dec!(1),
        )
        .registry
        .execute_expect_failure(false);

        self
    }

    pub fn flash_loan_address(&mut self) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            self.pool_address.unwrap(),
            "flash_loan_address",
            manifest_args!(),
        );
        self.registry
            .env
            .new_instruction("flash_loan_address", 1, 0);
        self
    }

    pub fn last_observation_index(&mut self) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            self.pool_address.unwrap(),
            "last_observation_index",
            manifest_args!(),
        );
        self.registry
            .env
            .new_instruction("last_observation_index", 1, 0);
        self
    }

    pub fn oldest_observation_at(&mut self) -> &mut FlexPoolTestHelper {
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder.call_method(
            self.pool_address.unwrap(),
            "oldest_observation_at",
            manifest_args!(),
        );
        self.registry
            .env
            .new_instruction("oldest_observation_at", 1, 0);
        self
    }

    pub fn jump_to_timestamp_seconds(&mut self, seconds: u64) {
        let current_time = self
            .registry
            .env
            .test_runner
            .get_current_time(TimePrecision::Minute)
            .seconds_since_unix_epoch as u64;
        if current_time == seconds {
            return;
        }

        let current_round = self
            .registry
            .env
            .test_runner
            .get_consensus_manager_state()
            .round
            .number();
        self.registry
            .env
            .test_runner
            .advance_to_round_at_timestamp(Round::of(current_round + 1), (seconds * 1000) as i64);
    }

    pub fn jump_to_timestamp_minutes(&mut self, minutes: u64) {
        self.jump_to_timestamp_seconds(minutes * 60);
    }

    pub fn a_address(&self) -> ResourceAddress {
        self.registry.env.a_address
    }

    pub fn b_address(&self) -> ResourceAddress {
        self.registry.env.b_address
    }

    pub fn x_address(&self) -> ResourceAddress {
        self.registry.env.x_address
    }

    pub fn y_address(&self) -> ResourceAddress {
        self.registry.env.y_address
    }

    pub fn v_address(&self) -> ResourceAddress {
        self.registry.env.v_address
    }

    pub fn u_address(&self) -> ResourceAddress {
        self.registry.env.u_address
    }

    pub fn j_nft_address(&self) -> ResourceAddress {
        self.registry.env.j_nft_address
    }

    pub fn k_nft_address(&self) -> ResourceAddress {
        self.registry.env.k_nft_address
    }

    pub fn admin_badge_address(&self) -> ResourceAddress {
        self.registry.env.admin_badge_address
    }

    pub fn add_liquidity_failure(&mut self, x_input: Decimal, y_input: Decimal) {
        self.add_liquidity_default(x_input, y_input)
            .registry
            .execute_expect_failure(false);
    }

    pub fn instantiate_with_liquidity_success(
        &mut self,
        x_input: Decimal,
        y_input: Decimal,
        input_fee_rate: Decimal,
        lp_amount_expected: Decimal,
        x_remainder_expected: Decimal,
        y_remainder_expected: Decimal,
    ) -> Vec<Vec<ResourceSpecifier>> {
        let receipt = self
            .instantiate_with_liquidity(
                self.x_address(),
                x_input,
                self.y_address(),
                y_input,
                input_fee_rate,
                self.registry.registry_address.unwrap(),
            )
            .registry
            .execute_expect_success(false);
        let commit_result = receipt.execution_receipt.expect_commit_success();
        let (pool_address, _): (ComponentAddress, Bucket) =
            receipt.outputs("instantiate_with_liquidity")[0];
        self.pool_address = Some(pool_address);
        self.lp_address = Some(commit_result.new_resource_addresses()[0]);
        self.liquidity_pool_address = Some(commit_result.new_component_addresses()[1]);
        let output_buckets = receipt.output_buckets("instantiate_with_liquidity");
        self.add_liquidity_success_internal(
            output_buckets,
            lp_amount_expected,
            x_remainder_expected,
            y_remainder_expected,
        )
    }

    pub fn add_liquidity_success(
        &mut self,
        x_input: Decimal,
        y_input: Decimal,
        lp_amount_expected: Decimal,
        x_remainder_expected: Decimal,
        y_remainder_expected: Decimal,
    ) -> Vec<Vec<ResourceSpecifier>> {
        let receipt = self
            .add_liquidity_default(x_input, y_input)
            .registry
            .execute_expect_success(false);
        let output_buckets = receipt.output_buckets("add_liquidity");
        self.add_liquidity_success_internal(
            output_buckets,
            lp_amount_expected,
            x_remainder_expected,
            y_remainder_expected,
        )
    }

    fn add_liquidity_success_internal(
        &mut self,
        output_buckets: Vec<Vec<ResourceSpecifier>>,
        lp_amount_expected: Decimal,
        x_remainder_expected: Decimal,
        y_remainder_expected: Decimal,
    ) -> Vec<Vec<ResourceSpecifier>> {
        self.lp_address = Some(output_buckets[0][0].address());
        let lp_expected = Amount(self.lp_address.unwrap(), lp_amount_expected);

        if x_remainder_expected == dec!(0) && y_remainder_expected == dec!(0) {
            assert_eq!(output_buckets, vec![vec![lp_expected]]);
            return output_buckets;
        }

        let remainder = if x_remainder_expected == dec!(0) {
            Amount(self.y_address(), y_remainder_expected)
        } else {
            Amount(self.x_address(), x_remainder_expected)
        };
        assert_eq!(output_buckets, vec![vec![lp_expected, remainder]]);

        output_buckets
    }

    pub fn swap_failure(&mut self, input_address: ResourceAddress, input_amount: Decimal) {
        self.swap(input_address, input_amount)
            .registry
            .execute_expect_failure(false);
    }

    pub fn swap_success(
        &mut self,
        input_address: ResourceAddress,
        input_amount: Decimal,
        output_amount_expected: Decimal,
    ) {
        let receipt = self
            .swap(input_address, input_amount)
            .registry
            .execute_expect_success(false);
        let output_buckets = receipt.output_buckets("swap");

        let output_address = if self.x_address() == input_address {
            self.y_address()
        } else {
            self.x_address()
        };

        assert_eq!(
            output_buckets,
            vec![vec![Amount(output_address, output_amount_expected)]],
            "\nInput: Address {:?} Amount {:?}, Output: Address {:?} Amount {:?}",
            input_address,
            input_amount,
            output_address,
            output_amount_expected
        );
    }

    pub fn remove_liquidity_success(
        &mut self,
        lp_amount: Decimal,
        x_output_expected: Decimal,
        y_output_expected: Decimal,
    ) {
        let receipt = self
            .remove_liquidity_default(lp_amount)
            .registry
            .execute_expect_success(false);
        let output_buckets = receipt.output_buckets("remove_liquidity");

        assert_eq!(
            output_buckets,
            vec![vec![
                Amount(self.x_address(), x_output_expected),
                Amount(self.y_address(), y_output_expected)
            ]],
            "\nX Amount = {:?}, Y Amount {:?}",
            x_output_expected,
            y_output_expected
        );
    }

    pub fn removable_liquidity_success(
        &mut self,
        lp_amount: Decimal,
        x_output_expected: Decimal,
        y_output_expected: Decimal,
    ) {
        let receipt = self
            .removable_liquidity(lp_amount)
            .registry
            .execute_expect_success(false);
        let outputs: Vec<IndexMap<ResourceAddress, Decimal>> =
            receipt.outputs("removable_liquidity");

        assert_eq!(
            outputs,
            vec![
                indexmap! { self.x_address() => x_output_expected, self.y_address() => y_output_expected }
            ],
            "\nX Amount = {:?}, Y Amount {:?}",
            x_output_expected,
            y_output_expected
        );
    }

    pub fn remove_liquidity_failure(&mut self, lp_amount: Decimal) {
        self.remove_liquidity_default(lp_amount)
            .registry
            .execute_expect_failure(false);
    }

    pub fn price_sqrt_success(&mut self, price_sqrt_expected: Option<PreciseDecimal>) {
        let receipt = self.price_sqrt().registry.execute_expect_success(false);
        let price_sqrt: Vec<Option<PreciseDecimal>> = receipt.outputs("price_sqrt");

        assert_eq!(price_sqrt, vec![price_sqrt_expected]);
    }

    pub fn display_meta(&mut self, key: &str) {
        let meta = self
            .registry
            .env
            .test_runner
            .get_metadata(self.lp_address.unwrap().into(), key)
            .unwrap();
        println!("{:?}", meta);
    }

    pub fn set_whitelist_registry(&mut self) -> &mut FlexPoolTestHelper {
        let registry_address = self.registry.registry_address.unwrap();
        self.set_metadata("registry_components", vec![registry_address])
    }

    pub fn set_whitelist_registry_value(
        &mut self,
        value: impl ToMetadataEntry,
    ) -> &mut FlexPoolTestHelper {
        self.set_metadata("registry_components", value)
    }

    pub fn lock_whitelist_registry(&mut self) -> &mut FlexPoolTestHelper {
        self.lock_metadata("registry_components")
    }

    pub fn set_whitelist_hook(&mut self, package_name: &str) -> &mut FlexPoolTestHelper {
        self.set_whitelist_packages("hook_packages", vec![package_name])
    }

    pub fn set_whitelist_hook_value(
        &mut self,
        value: impl ToMetadataEntry,
    ) -> &mut FlexPoolTestHelper {
        self.set_metadata("hook_packages", value)
    }

    pub fn lock_whitelist_hook(&mut self) -> &mut FlexPoolTestHelper {
        self.lock_metadata("hook_packages")
    }

    pub fn set_whitelist_packages(
        &mut self,
        metadata_key: &str,
        package_names: Vec<&str>,
    ) -> &mut FlexPoolTestHelper {
        let global_package_addresses: Vec<GlobalAddress> = package_names
            .iter()
            .map(|package_name| self.registry.env.package_address(package_name).into())
            .collect();
        self.set_metadata(metadata_key, global_package_addresses)
    }

    pub fn set_metadata(
        &mut self,
        key: impl Into<String>,
        value: impl ToMetadataEntry,
    ) -> &mut FlexPoolTestHelper {
        let precision_pool_package_address: GlobalAddress =
            self.registry.env.package_address("flex_pool").into();
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .create_proof_from_account_of_amount(
                self.registry.env().account,
                self.admin_badge_address(),
                dec!(1),
            )
            .set_metadata(precision_pool_package_address, key, value);
        self.registry.env.new_instruction("set_metadata", 2, 1);
        self
    }

    pub fn lock_metadata(&mut self, key: impl Into<String>) -> &mut FlexPoolTestHelper {
        let precision_pool_package_address: GlobalAddress =
            self.registry.env.package_address("flex_pool").into();
        let manifest_builder = mem::take(&mut self.registry.env.manifest_builder);
        self.registry.env.manifest_builder = manifest_builder
            .create_proof_from_account_of_amount(
                self.registry.env().account,
                self.admin_badge_address(),
                dec!(1),
            )
            .lock_metadata(precision_pool_package_address, key);
        self.registry.env.new_instruction("lock_metadata", 2, 1);
        self
    }
}

pub trait AttoDecimal {
    const ATTO: Decimal;
}

impl AttoDecimal for Decimal {
    const ATTO: Self = Self(I192::ONE);
}

pub fn swap_with_hook_action_test(
    method_name: &str,
    before_swap_amount: Option<Decimal>,
    after_swap_amount: Option<Decimal>,
    expect_success: bool,
) {
    let packages: HashMap<&str, &str> = vec![
        ("registry", "registry"),
        ("flex_pool", "."),
        ("test_hook", "test_hook"),
    ]
    .into_iter()
    .collect();
    let mut helper = FlexPoolTestHelper::new_with_packages(packages, true);
    helper.set_whitelist_registry();
    helper.set_whitelist_hook("test_hook");

    let package_address = helper.registry.env.package_address("test_hook");
    let manifest_builder = mem::replace(
        &mut helper.registry.env.manifest_builder,
        ManifestBuilder::new(),
    );
    helper.registry.env.manifest_builder = manifest_builder.call_function(
        package_address,
        "TestSwapHook",
        "instantiate",
        manifest_args!(helper.x_address(), helper.y_address()),
    );
    helper
        .registry
        .env
        .new_instruction("instantiate_test_hook", 1, 0);

    let receipt = helper.registry.execute_expect_success(false);

    let new_resource_ads = receipt
        .execution_receipt
        .expect_commit_success()
        .new_resource_addresses();

    let outputs: Vec<(ComponentAddress, Bucket)> = receipt.outputs("instantiate_test_hook");

    let hook_address = outputs[0].0;
    let hook_badge_address = new_resource_ads[0];

    let hook_infos = vec![(hook_address, hook_badge_address)];

    helper.instantiate_default_with_hooks(hook_infos, false);
    helper
        .add_liquidity_default(dec!(1), dec!(1))
        .registry
        .execute_expect_success(false);
    let manifest_builder = mem::replace(
        &mut helper.registry.env.manifest_builder,
        ManifestBuilder::new(),
    );
    helper.registry.env.manifest_builder = manifest_builder.call_method(
        hook_address,
        method_name,
        manifest_args!(before_swap_amount, after_swap_amount),
    );
    helper.registry.execute_expect_success(false);

    helper.swap(helper.y_address(), dec!(1));

    if expect_success {
        helper.registry.execute_expect_success(false);
    } else {
        helper.registry.execute_expect_failure(false);
    }
}
