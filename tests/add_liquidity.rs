use flex_pool_test_helper::*;
use scrypto::prelude::*;
use scrypto_testenv::*;

fn instantiate_helper() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
}

fn add_liquidity_expect_failure(
    helper: Option<FlexPoolTestHelper>,
    x_input: Decimal,
    y_input: Decimal,
) {
    let mut helper = helper.unwrap_or(instantiate_helper());
    helper.add_liquidity_failure(x_input, y_input);
}

fn add_liquidity_expect_success(
    helper: Option<FlexPoolTestHelper>,
    x_input: Decimal,
    y_input: Decimal,
    lp_amount_expected: Decimal,
    x_output_expected: Decimal,
    y_output_expected: Decimal,
) -> FlexPoolTestHelper {
    let mut helper = helper.unwrap_or(instantiate_helper());
    helper.add_liquidity_success(
        x_input,
        y_input,
        lp_amount_expected,
        x_output_expected,
        y_output_expected,
    );
    helper
}

#[test]
fn test_add_liquidity_invalid_token_both() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.u_address(), dec!(1), helper.v_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_add_liquidity_invalid_token_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.u_address(), dec!(1), helper.v_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_add_liquidity_invalid_token_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .add_liquidity(helper.x_address(), dec!(1), helper.v_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_add_liquidity_symmetric() {
    add_liquidity_expect_success(None, dec!(1), dec!(1), dec!(1), dec!(0), dec!(0));
}

#[test]
fn test_add_liquidity_zero_amount() {
    add_liquidity_expect_failure(None, dec!(0), dec!(0));
}

#[test]
fn test_add_liquidity_liquidity_amount_too_low_for_x() {
    add_liquidity_expect_success(
        None,
        dec!(0),
        dec!(0.000000000000000001),
        dec!(0.000000000000000001),
        dec!(0),
        dec!(0),
    );
}

#[test]
fn test_add_liquidity_liquidity_amount_too_low_for_y() {
    add_liquidity_expect_success(
        None,
        dec!(0.000000000000000001),
        dec!(0),
        dec!(0.000000000000000001),
        dec!(0),
        dec!(0),
    );
}

#[test]
fn test_add_liquidity_minimum_possible_liquidity() {
    add_liquidity_expect_success(
        None,
        dec!("0.000000000000000001"),
        dec!("0.000000000000000001"),
        dec!("0.000000000000000001"),
        dec!(0),
        dec!(0),
    );
}

#[test]
fn test_add_liquidity_maximum_possible_liquidity() {
    add_liquidity_expect_success(
        None,
        MAX_SUPPLY,
        MAX_SUPPLY,
        dec!("5708990770823839524233143877.797980545530986496"),
        Decimal::ZERO,
        Decimal::ZERO,
    );
}

#[test]
fn test_add_liquidity_multiple_positions() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper =
        add_liquidity_expect_success(Some(helper), dec!(1), dec!(1), dec!(1), dec!(0), dec!(0));
    add_liquidity_expect_success(Some(helper), dec!(2), dec!(2), dec!(2), dec!(0), dec!(0));
}

#[test]
fn test_add_liquidity_with_remainder() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper =
        add_liquidity_expect_success(Some(helper), dec!(1), dec!(1), dec!(1), dec!(0), dec!(0));
    add_liquidity_expect_success(Some(helper), dec!(3), dec!(2), dec!(2), dec!(1), dec!(0));
}
