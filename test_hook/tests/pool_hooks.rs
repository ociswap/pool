mod helper;

#[cfg(test)]
mod test_test_hook {
    use super::*;
    use helper::HookTestTestHelper;
    use ociswap_pool_hooks::HookCall;
    use scrypto::prelude::*;
    use test_hook::test_hook::TestAccess;

    #[test]
    fn test_all_calls_all_accesses() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook
        let mut hooks = Vec::new();
        let mut calls = Vec::new();
        calls.push(HookCall::BeforeInstantiate);
        calls.push(HookCall::AfterInstantiate);
        calls.push(HookCall::BeforeSwap);
        calls.push(HookCall::AfterSwap);

        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_before_instantiate_call() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook with only BeforeInstantiate call
        let mut hooks = Vec::new();
        let calls = vec![HookCall::BeforeInstantiate];
        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_after_instantiate_call() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook with only AfterInstantiate call
        let mut hooks = Vec::new();
        let calls = vec![HookCall::AfterInstantiate];
        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_before_swap_call() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook with only BeforeSwap call
        let mut hooks = Vec::new();
        let calls = vec![HookCall::BeforeSwap];
        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_after_swap_call() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook with only AfterSwap call
        let mut hooks = Vec::new();
        let calls = vec![HookCall::AfterSwap];
        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_all_calls_without_hooks() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook
        let mut hooks = Vec::new();
        let calls = Vec::new();

        let access = TestAccess::new();

        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    #[test]
    fn test_all_calls_with_two_same_hooks() {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook
        let mut hooks = Vec::new();
        let mut calls = Vec::new();

        calls.push(HookCall::BeforeInstantiate);
        calls.push(HookCall::AfterInstantiate);
        calls.push(HookCall::BeforeSwap);
        calls.push(HookCall::AfterSwap);

        let access = TestAccess::new();

        // First test hook
        hooks.push(hook_helper.instantiate_test_hook_output(calls.clone(), access.clone()));
        // Second test hook
        hooks.push(hook_helper.instantiate_test_hook_output(calls, access));

        hook_helper.execute_all_calls(hooks);
    }

    fn hook_default_helper() -> (HookTestTestHelper, Vec<(ComponentAddress, ResourceAddress)>) {
        let mut hook_helper = HookTestTestHelper::new();

        // Instantiate hook
        let calls = vec![HookCall::BeforeInstantiate, HookCall::AfterInstantiate];
        let access = TestAccess::new();
        let hooks = vec![hook_helper.instantiate_test_hook_output(calls, access)];

        (hook_helper, hooks)
    }

    #[test]
    #[should_panic]
    fn test_hook_whiteliste_missing() {
        let (mut hook_helper, hooks) = hook_default_helper();
        hook_helper.pool.set_whitelist_registry();
        // Do not whitelist the current "TestHook" in package "test_hook"

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    #[should_panic]
    fn test_hook_whitelist_empty_vec() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper
            .pool
            .set_whitelist_hook_value(Vec::<GlobalAddress>::new());

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    #[should_panic]
    fn test_hook_whitelist_other_value_type() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper.pool.set_whitelist_hook_value("OTHER");

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    #[should_panic]
    fn test_hook_whitelist_other_value_vec_type() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper.pool.set_whitelist_hook_value(vec!["OTHER"]);

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    #[should_panic]
    fn test_hook_whitelist_other_package_address() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper.pool.set_whitelist_hook_value(vec![hook_helper
            .pool
            .registry
            .env
            .package_address("registry")]);

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    fn test_hook_whitelist_two_package_addresses_hook_and_other() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper.pool.set_whitelist_hook_value(vec![
            hook_helper.pool.registry.env.package_address("test_hook"),
            hook_helper.pool.registry.env.package_address("registry"),
        ]);

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    fn test_hook_whitelist_two_same_hook_package_addresses() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();
        hook_helper.pool.set_whitelist_hook_value(vec![
            hook_helper.pool.registry.env.package_address("test_hook"),
            hook_helper.pool.registry.env.package_address("test_hook"),
        ]);

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }

    #[test]
    fn test_hook_whitelist_two_package_addresses_hook_and_resource() {
        let (mut hook_helper, hooks) = hook_default_helper();

        hook_helper.pool.set_whitelist_registry();

        let global_addresses: Vec<GlobalAddress> = vec![
            hook_helper
                .pool
                .registry
                .env
                .package_address("test_hook")
                .into(),
            hook_helper.pool.registry.env.x_address.into(),
        ];
        hook_helper.pool.set_whitelist_hook_value(global_addresses);

        hook_helper
            .pool
            .instantiate_default_with_hooks(hooks, false);
    }
}
