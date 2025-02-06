use ociswap_pool_hooks::HookCall;
use ociswap_pool_test_helper::*;
use radix_transactions::prelude::ManifestBuilder;
use scrypto::prelude::*;
use scrypto_testenv::*;
use std::mem;
use test_hook::test_hook::TestAccess;

pub struct HookTestTestHelper {
    pub pool: PoolTestHelper,

    pub hook_address: Option<ComponentAddress>,
    pub admin_badge_address: Option<ResourceAddress>,
}

impl TestHelperExecution for HookTestTestHelper {
    fn env(&mut self) -> &mut TestEnvironment {
        &mut self.pool.registry.env
    }
}

impl HookTestTestHelper {
    pub fn new() -> HookTestTestHelper {
        let packages: HashMap<&str, &str> = vec![
            ("registry", "../registry"),
            ("pool", ".."),
            ("test_hook", "."),
        ]
        .into_iter()
        .collect();
        let pool = PoolTestHelper::new_with_packages(packages, true);

        Self {
            pool,
            hook_address: None,
            admin_badge_address: None,
        }
    }

    pub fn instantiate_test_hook(
        &mut self,
        calls: Vec<HookCall>,
        calls_access: TestAccess,
    ) -> &mut HookTestTestHelper {
        let package_address = self.env().package_address("test_hook");
        let manifest_builder =
            mem::replace(&mut self.env().manifest_builder, ManifestBuilder::new());
        self.env().manifest_builder = manifest_builder.call_function(
            package_address,
            "TestHookBasicPool",
            "instantiate",
            manifest_args!(
                calls,
                calls_access,
                self.pool.registry.x_address(),
                self.pool.registry.y_address()
            ),
        );
        self.env().new_instruction("instantiate_test_hook", 1, 0);
        self
    }

    pub fn instantiate_test_hook_output(
        &mut self,
        calls: Vec<HookCall>,
        calls_access: TestAccess,
    ) -> (ComponentAddress, ResourceAddress) {
        self.instantiate_test_hook(calls, calls_access);

        let receipt = self.execute(false);

        let new_resource_ads = receipt
            .execution_receipt
            .expect_commit_success()
            .new_resource_addresses();

        let hook_badge_address: Vec<(ComponentAddress, Bucket)> =
            receipt.outputs("instantiate_test_hook");
        (hook_badge_address[0].0, new_resource_ads[0])
    }

    pub fn execute_all_calls(&mut self, hooks: Vec<(ComponentAddress, ResourceAddress)>) {
        self.pool.set_whitelist_registry();
        self.pool.set_whitelist_hook("test_hook");
        // Instantiate pool
        self.pool.instantiate_default_with_hooks(hooks, false);

        // Add liquidity
        self.pool.add_liquidity_default_execute(dec!(10), dec!(10));

        // First Swap
        self.pool.swap(self.pool.registry.x_address(), dec!(1));
        self.execute_expect_success(false);

        // Elpasing time
        advance_to_second_in_round(&mut &mut self.pool, 360);

        // Second Swap
        self.pool
            .swap(self.pool.registry.y_address(), dec!("0.859215564276274065"));
        self.execute_expect_success(false);

        // Remove liquidity
        self.pool.remove_liquidity_success(
            dec!(1),
            dec!("1.005012534670974519"),
            dec!("0.995012465518536497"),
        );
    }
}

fn advance_to_second_in_round(pool_helper: &mut PoolTestHelper, second: i64) {
    let current_round = pool_helper
        .registry
        .env
        .test_runner
        .get_consensus_manager_state()
        .round
        .number();
    pool_helper
        .registry
        .env
        .test_runner
        .advance_to_round_at_timestamp(Round::of(current_round + 1), second * 1000);
}
