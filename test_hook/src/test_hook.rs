use common::pools::SwapType;
use ociswap_pool_hooks::*;
use scrypto::prelude::*;

#[blueprint]
mod test_hook_simple_pool {
    enable_method_auth! {
        roles {
            hook_admin => updatable_by: [OWNER];
        },
        methods {
            calls => PUBLIC;
            before_instantiate => restrict_to: [hook_admin];
            after_instantiate => restrict_to: [hook_admin];
            before_swap => restrict_to: [hook_admin];
            after_swap => restrict_to: [hook_admin];
        }
    }
    struct TestHookPool {
        calls: Vec<HookCall>,
        calls_access: TestAccess,
        x_vault: Vault,
        y_vault: Vault,
    }

    impl TestHookPool {
        pub fn instantiate(
            calls: Vec<HookCall>,
            calls_access: TestAccess,
            x_address: ResourceAddress,
            y_address: ResourceAddress,
        ) -> (Global<TestHookPool>, FungibleBucket) {
            let hook_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                        "name" => "Hook badge", locked;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(allow_all);
                    minter_updater => rule!(deny_all);
                ))
                .mint_initial_supply(1);

            debug!("{:?}", hook_badge.resource_address());
            let hook_badge_address = hook_badge.resource_address();

            let x_vault: Vault = Vault::new(x_address);
            let y_vault: Vault = Vault::new(y_address);

            let hook_component = (Self {
                calls,
                calls_access,
                x_vault,
                y_vault,
            })
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                hook_admin => rule!(require(hook_badge_address));
            ))
            .globalize();

            (hook_component, hook_badge)
        }

        pub fn calls(&mut self) -> Vec<HookCall> {
            self.calls.clone()
        }

        pub fn before_instantiate(
            &mut self,
            state: BeforeInstantiateState,
        ) -> (BeforeInstantiateState,) {
            (state,)
        }

        pub fn after_instantiate(
            &mut self,
            state: AfterInstantiateState,
        ) -> (AfterInstantiateState,) {
            (state,)
        }

        pub fn before_swap(
            &mut self,
            before_swap_state: BeforeSwapState,
            mut input_bucket: Bucket,
        ) -> (BeforeSwapState, Bucket) {
            debug!("[TEST HOOK SIMPLE POOL] before_swap");
            if self.calls_access.before_swap_input {
                debug!("[TEST HOOK SIMPLE POOL] before_swap: In before_swap_input");
                match before_swap_state.swap_type {
                    SwapType::BuyX => {
                        self.y_vault.put(input_bucket.take(Decimal::ZERO));
                    }
                    SwapType::SellX => {
                        self.x_vault.put(input_bucket.take(Decimal::ZERO));
                    }
                }
            }
            (before_swap_state, input_bucket)
        }

        pub fn after_swap(
            &mut self,
            after_swap_state: AfterSwapState,
            mut output_bucket: Bucket,
        ) -> (AfterSwapState, Bucket) {
            debug!("[TEST HOOK SIMPLE POOL] after_swap");
            if self.calls_access.after_swap_output {
                debug!("[TEST HOOK SIMPLE POOL] after_swap: In after_swap_output");
                match after_swap_state.swap_type {
                    SwapType::BuyX => {
                        self.x_vault.put(output_bucket.take(Decimal::ZERO));
                    }
                    SwapType::SellX => {
                        self.y_vault.put(output_bucket.take(Decimal::ZERO));
                    }
                }
            }
            (after_swap_state, output_bucket)
        }
    }
}

#[derive(ScryptoSbor, Clone, Debug, ManifestSbor)]
pub struct TestAccess {
    pub before_swap_input: bool,
    pub after_swap_output: bool,
}

impl TestAccess {
    pub fn new() -> Self {
        Self {
            before_swap_input: true,
            after_swap_output: true,
        }
    }
}
