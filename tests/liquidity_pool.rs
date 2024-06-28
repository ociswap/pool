use flex_pool_test_helper::*;
use scrypto::prelude::*;
use scrypto_testenv::*;

#[test]
fn test_redeem() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .redeem(helper.lp_address.unwrap(), dec!(1))
        .registry
        .execute_expect_success(false);
}

#[test]
fn test_redeem_wrong_address() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .redeem(helper.x_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_redeem_all_then_remove_liquidity() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .redeem(helper.lp_address.unwrap(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.remove_liquidity_failure(dec!(1));
}

#[test]
fn test_contribute() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .contribute(helper.x_address(), dec!(1), helper.y_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_protected_deposit() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .protected_deposit(helper.x_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_protected_withdraw() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .protected_withdraw(helper.x_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}
