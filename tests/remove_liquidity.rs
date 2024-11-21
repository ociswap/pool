use flex_pool_test_helper::*;
use scrypto::prelude::*;
use scrypto_testenv::TestHelperExecution;

#[test]
fn test_remove_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.remove_liquidity_success(dec!(1), dec!(1), dec!(1));
}

#[test]
fn test_remove_liquidity_full() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.remove_liquidity_success(dec!(10), dec!(10), dec!(10));
}

#[test]
fn test_remove_liquidity_after_swap() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    // checking lp amount with add_liquidity_success
    helper.add_liquidity_success(dec!(10), dec!(10), dec!(10), dec!(0), dec!(0));
    helper.swap_success(helper.x_address(), dec!(1), dec!(0.90909090909090909));
    helper.remove_liquidity_success(
        Decimal::ONE,
        dec!(1.1),
        dec!(0.90909090909090909) + Decimal::ATTO, // TODO
    );
}

#[test]
fn test_remove_full_liquidity_after_swap() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.swap_success(helper.x_address(), dec!(1), dec!(0.90909090909090909));
    helper.remove_liquidity_success(
        dec!(10),
        dec!(11),
        dec!(9.090909090909090909) + Decimal::ATTO, // we do have vaults empty
    );
}

#[test]
fn test_remove_liquidity_wrong_token_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper
        .remove_liquidity(helper.x_address(), Decimal::ONE)
        .registry
        .execute_expect_failure(false);
}
#[test]
fn test_remove_liquidity_wrong_token_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper
        .remove_liquidity(helper.y_address(), Decimal::ONE)
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_remove_atto_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper.swap_success(helper.x_address(), dec!(1), dec!(0.90909090909090909));
    helper.remove_liquidity_success(Decimal::ATTO, Decimal::ATTO, dec!(0.0));
}

#[test]
fn test_remove_atto_liquidity_then_full_remove() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper.swap_success(helper.x_address(), dec!(1), dec!(0.90909090909090909));
    helper.remove_liquidity_success(Decimal::ATTO, Decimal::ATTO, dec!(0));
    helper.remove_liquidity_success(
        dec!(10) - Decimal::ATTO,
        dec!(11) - Decimal::ATTO,
        dec!(9.090909090909090909) + Decimal::ATTO, // we do have vaults empty
    );
}

#[test]
fn test_remove_too_much_liquidity() {
    // We'll do the test even if we can't have more lp tokens
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper.swap_success(helper.x_address(), dec!(1), dec!(0.90909090909090909));
    helper.remove_liquidity_failure(dec!(10) + Decimal::ATTO);
}

#[test]
fn test_remove_max_decimal_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.remove_liquidity_failure(Decimal::MAX);
}

#[test]
fn test_remove_liquidity_after_max_swap() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10));
    helper.swap_success(
        helper.x_address(),
        dec!(5708990770823839524233143867.797980545530986496), //(MAX_SUPPLY-10)
        dec!(9.999999999999999999),
    );
    helper.remove_liquidity_success(
        dec!(10),
        dec!(5708990770823839524233143877.797980545530986496),
        dec!(0.0) + Decimal::ATTO, // we do have vaults empty
    );
}

#[test]
fn test_removable_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(10), helper.y_address(), dec!(10))
        .registry
        .execute_expect_success(false);
    helper.removable_liquidity_success(dec!(10), dec!(10), dec!(10));
}
