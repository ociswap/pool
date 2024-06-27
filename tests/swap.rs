use flex_pool::constants::*;
use scrypto::prelude::*;
use scrypto_testenv::MAX_SUPPLY;
use test_case::test_case;
use flex_pool_test_helper::*;

fn add_liquidity_success(mut helper: FlexPoolTestHelper) -> FlexPoolTestHelper {
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper
}

fn instantiate_helper() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    add_liquidity_success(helper)
}

fn instantiate_helper_with_fees() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_input_fee(dec!("0.02"), false);
    add_liquidity_success(helper)
}

fn instantiate_helper_with_atto_fees() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_input_fee(Decimal::ATTO, false);
    add_liquidity_success(helper)
}

fn instantiate_helper_with_max_percent_protocol_fees() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!("0.05"), dec!(0.25), dec!(0.5), false);
    add_liquidity_success(helper)
}

fn swap_expect_failure(
    helper: Option<FlexPoolTestHelper>,
    input_address: Option<ResourceAddress>,
    input_amount: Decimal,
) {
    let mut helper = helper.unwrap_or(instantiate_helper());
    helper.swap_failure(input_address.unwrap_or(helper.x_address()), input_amount);
}

fn swap_buy_x_expect_failure(helper: FlexPoolTestHelper, input_amount: Decimal) {
    let y_address = helper.y_address();
    swap_expect_failure(Some(helper), Some(y_address), input_amount);
}

fn swap_sell_x_expect_failure(helper: FlexPoolTestHelper, input_amount: Decimal) {
    let x_address = helper.x_address();
    swap_expect_failure(Some(helper), Some(x_address), input_amount);
}

fn swap_expect_success(
    helper: Option<FlexPoolTestHelper>,
    input_address: Option<ResourceAddress>,
    input_amount: Decimal,
    output_amount_expected: Decimal,
) -> FlexPoolTestHelper {
    let mut helper = helper.unwrap_or(instantiate_helper());
    helper.swap_success(
        input_address.unwrap_or(helper.x_address()),
        input_amount,
        output_amount_expected,
    );
    helper
}

fn swap_buy_x_expect_success(
    helper: FlexPoolTestHelper,
    input_amount: Decimal,
    output_amount_expected: Decimal,
) {
    let y_address = helper.y_address();
    swap_expect_success(
        Some(helper),
        Some(y_address),
        input_amount,
        output_amount_expected,
    );
}

fn swap_sell_x_expect_success(
    helper: FlexPoolTestHelper,
    input_amount: Decimal,
    output_amount_expected: Decimal,
) {
    let x_address = helper.x_address();
    swap_expect_success(
        Some(helper),
        Some(x_address),
        input_amount,
        output_amount_expected,
    );
}

fn x_address() -> ResourceAddress {
    FlexPoolTestHelper::new().x_address()
}

fn y_address() -> ResourceAddress {
    FlexPoolTestHelper::new().y_address()
}

#[test]
fn test_swap_sell_x() {
    swap_expect_success(
        None,
        Some(x_address()),
        dec!(10),
        dec!("0.499950004999500049"),
    );
}

#[test]
fn test_swap_buy_x() {
    swap_expect_success(
        None,
        Some(y_address()),
        dec!(10),
        dec!("199.600798403193612774"),
    );
}

#[test]
fn test_swap_sell_x_minimum_amounts() {
    swap_expect_success(None, Some(x_address()), Decimal::ZERO, Decimal::ZERO);
    swap_expect_success(None, Some(x_address()), Decimal::ATTO, Decimal::ZERO);
    swap_expect_success(
        None,
        Some(x_address()),
        dec!("0.000000000000000020"),
        Decimal::ZERO,
    );
    swap_expect_success(
        None,
        Some(x_address()),
        dec!("0.000000000000000021"),
        Decimal::ATTO,
    );
}

#[test]
fn test_swap_sell_x_minimum_amounts_with_fees() {
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(x_address()),
        Decimal::ZERO,
        Decimal::ZERO,
    );
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(x_address()),
        Decimal::ATTO,
        Decimal::ZERO,
    );
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(x_address()),
        dec!("0.000000000000000021"),
        Decimal::ZERO,
    );
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(x_address()),
        dec!("0.000000000000000022"),
        Decimal::ATTO,
    );
}

#[test]
fn test_swap_buy_x_minimum_amounts() {
    swap_expect_success(None, Some(y_address()), Decimal::ZERO, Decimal::ZERO);
    swap_expect_success(
        None,
        Some(y_address()),
        Decimal::ATTO,
        dec!("0.000000000000000019"),
    );
    swap_expect_success(
        None,
        Some(y_address()),
        dec!("0.000000000000000002"),
        dec!("0.000000000000000039"),
    );
}

#[test]
fn test_swap_buy_x_minimum_amounts_with_fees() {
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(y_address()),
        Decimal::ZERO,
        Decimal::ZERO,
    );
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(y_address()),
        Decimal::ATTO,
        Decimal::ZERO,
    );
    swap_expect_success(
        Some(instantiate_helper_with_fees()),
        Some(y_address()),
        dec!("0.000000000000000002"),
        dec!("0.000000000000000019"),
    );
}

#[test]
fn test_swap_sell_x_with_atto_fees() {
    swap_expect_success(
        Some(instantiate_helper_with_atto_fees()),
        Some(x_address()),
        Decimal::ONE,
        dec!("0.049999500004999949"),
    );

    swap_expect_success(
        Some(instantiate_helper_with_atto_fees()),
        Some(x_address()),
        Decimal::ATTO,
        Decimal::ZERO,
    );
}

#[test]
fn test_swap_buy_x_with_atto_fees() {
    swap_expect_success(
        Some(instantiate_helper_with_atto_fees()),
        Some(y_address()),
        Decimal::ONE,
        dec!("19.996000799840031973"),
    );

    swap_expect_success(
        Some(instantiate_helper_with_atto_fees()),
        Some(y_address()),
        Decimal::ATTO,
        dec!("0.000000000000000019") - dec!("0.000000000000000019"), // because ceiling input fee to Atto
    );
}

#[test]
fn swap_lp_fees_zero() {
    let mut helper = swap_expect_success(None, None, dec!(10), dec!("0.499950004999500049"));
    helper.remove_liquidity_success(
        dec!("22360.679774997896964092"),
        dec!("100010"),
        dec!("4999.500049995000499950") + Decimal::ATTO,
    )
}

#[test]
fn swap_lp_fees() {
    let mut helper = swap_expect_success(
        Some(instantiate_helper_with_fees()),
        None,
        dec!(10),
        dec!("0.489951984705498861"),
    );
    helper.remove_liquidity_success(
        dec!("22360.679774997896964092"),
        dec!("100009.98"),
        dec!("4999.510048015294501138") + Decimal::ATTO,
    )
}

#[test]
fn test_swap_liquidity_zero() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.swap_failure(helper.x_address(), dec!(1));
}

#[test]
fn swap_with_min_x_min_y_liquidity_sell_x_min_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    swap_sell_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_min_y_liquidity_sell_x_max_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_sell_x_expect_success(helper, MAX_SUPPLY - Decimal::ATTO, Decimal::ZERO);
}

#[test]
fn swap_with_min_x_min_y_liquidity_buy_x_min_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    swap_buy_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_min_y_liquidity_buy_x_max_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    // use the rest of Y not in the pool already
    swap_buy_x_expect_success(helper, MAX_SUPPLY - Decimal::ATTO, Decimal::ZERO);
}

#[test]
fn swap_with_min_x_max_y_liquidity_sell_x_min_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    // expect half the max supply of Y back
    swap_sell_x_expect_success(helper, Decimal::ATTO, MAX_SUPPLY / 2);
}

#[test]
fn swap_with_min_x_max_y_liquidity_sell_x_max_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_sell_x_expect_failure(helper, MAX_SUPPLY - Decimal::ATTO);
}

#[test]
fn swap_with_min_x_max_y_liquidity_buy_x_min_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY - Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_buy_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_max_y_liquidity_buy_x_max_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY - Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_buy_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_sell_x_min_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_sell_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_sell_x_max_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_sell_x_expect_success(helper, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_buy_x_min_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    // expect half the max supply of X back
    swap_buy_x_expect_success(helper, Decimal::ATTO, MAX_SUPPLY / 2);
}

#[test]
fn swap_with_max_x_min_y_liquidity_buy_x_max_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    // use the rest of Y not in the pool already
    swap_buy_x_expect_failure(helper, MAX_SUPPLY - Decimal::ATTO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_sell_x_min_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    swap_sell_x_expect_success(helper, Decimal::ATTO, Decimal::ATTO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_sell_x_max_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_sell_x_expect_success(helper, Decimal::ATTO, Decimal::ATTO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_buy_x_min_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY,
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // expect one attos of X
    swap_buy_x_expect_success(helper, Decimal::ATTO, Decimal::ATTO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_buy_x_max_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        MAX_SUPPLY,
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // use the rest of Y not in the pool already, expect one attos of X
    swap_buy_x_expect_success(helper, Decimal::ATTO, Decimal::ATTO);
}

#[test]
fn test_swap_instantiate_with_liquidity_with_fees() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate_with_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("0.02"),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    swap_sell_x_expect_success(helper, dec!(10), dec!("0.489951984705498861"));
}

#[test]
fn swap_lp_fees_max_percent_protocol() {
    let mut helper = swap_expect_success(
        Some(instantiate_helper_with_max_percent_protocol_fees()),
        None,
        dec!(10),
        dec!("0.474954879286467785"),
    );
    helper.remove_liquidity_success(
        dec!("22360.679774997896964092"),
        dec!("100009.875"),
        dec!("4999.525045120713532214") + Decimal::ATTO,
    )
}

#[test]
fn test_lower_divisibility() {
    let mut helper = FlexPoolTestHelper::new();
    let stable_address = helper.registry.env.test_runner.create_fungible_resource(
        dec!(10000000),
        6,
        helper.registry.env.account,
    );
    helper.registry.env.x_address = stable_address;
    helper.registry.env.y_address = XRD;
    helper.instantiate_with_liquidity_success(
        dec!(10),
        dec!(250),
        dec!(0.1),
        dec!(50),
        dec!(0),
        dec!(0),
    );
    // take output amount
    helper.swap_success(XRD, dec!(10), dec!(0.34749));
    // take protocol fees from input
    helper.swap_success(stable_address, dec!(1.929392), dec!(39.626427307422748447));
}

#[test]
fn swap_multiple() {
    let mut helper: FlexPoolTestHelper = swap_expect_success(
        None,
        Some(x_address()),
        dec!(10),
        dec!("0.499950004999500049"),
    );
    helper.swap_success(helper.x_address(), dec!(1), dec!("0.04998950165476798"));
}

// Test hook input_fee_rate in bounds
#[test_case(Some(dec!(0)), None, true ; "1")]
#[test_case(Some(INPUT_FEE_RATE_MAX / 2), None, true ; "2")]
#[test_case(Some(INPUT_FEE_RATE_MAX), None, true ; "3")]
#[test_case(Some(-Decimal::ATTO), None, false ; "4")]
#[test_case(Some(INPUT_FEE_RATE_MAX + Decimal::ATTO), None, false ; "5")]
#[test_case(None, Some(dec!(0)), true ; "6")]
#[test_case(None, Some(INPUT_FEE_RATE_MAX / 2), true ; "7")]
#[test_case(None, Some(INPUT_FEE_RATE_MAX), true ; "8")]
#[test_case(None, Some(-Decimal::ATTO), false ; "9")]
#[test_case(None, Some(INPUT_FEE_RATE_MAX + Decimal::ATTO), false ; "10")]
fn test_swap_hook_provided_input_fee_rate(
    before_swap_rate: Option<Decimal>,
    after_swap_rate: Option<Decimal>,
    expect_success: bool,
) {
    swap_with_hook_action_test(
        "set_input_fee_rates",
        before_swap_rate,
        after_swap_rate,
        expect_success,
    );
}

// Test hook bucket amount in bounds
#[test_case(Some(-dec!(0.1)), None, false; "negative")]
#[test_case(Some(dec!(0)), None, false; "0")]
#[test_case(Some(dec!(0.8)), None, false; "1")]
#[test_case(Some(dec!(0.89)), None, false; "2")]
#[test_case(Some(dec!(0.9)), None, true; "3")]
#[test_case(Some(dec!(1)), None, true; "4")]
#[test_case(None, Some(dec!(0.8)), false; "5")]
#[test_case(None, Some(dec!(0.89)), false; "6")]
#[test_case(None, Some(dec!(0.9)), true; "7")]
#[test_case(None, Some(dec!(1)), true; "8")]
#[test_case(Some(dec!(1.01)), None, false; "9")]
#[test_case(None, Some(dec!(1.01)), false; "10")]
fn test_swap_hook_returned_buckets(
    before_swap_rate: Option<Decimal>,
    after_swap_rate: Option<Decimal>,
    expect_success: bool,
) {
    swap_with_hook_action_test(
        "set_bucket_returned_fractions",
        before_swap_rate,
        after_swap_rate,
        expect_success,
    );
}
