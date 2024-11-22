use common::pools::SwapType;
use flex_pool_test_helper::*;
use scrypto::prelude::*;
use scrypto_testenv::MAX_SUPPLY;

#[test]
fn test_swap_sell_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.x_address(), dec!(10), dec!("0.499950004999500049"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), false);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.x_address(), dec!(10), dec!("0.49996875156192"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(20000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.x_address(), dec!(10), dec!("0.49996875234156"));
}

#[test]
fn test_swap_buy_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.y_address(), dec!(10), dec!("199.600798403193612774"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), false);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.y_address(), dec!(10), dec!("199.7503743916192"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(20000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.y_address(), dec!(10), dec!("199.7502497814148"));
}

#[test]
fn test_price_sqrt() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127623")));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), false);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127622")));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(20000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127623")));
}

fn instantiate_helper() -> FlexPoolTestHelper {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), false);
    helper
}

fn instantiate_helper_with_liquidity() -> FlexPoolTestHelper {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper
}

fn swap_expect_success(
    helper: Option<FlexPoolTestHelper>,
    swap_type: SwapType,
    input_amount: Decimal,
    output_amount_expected: Decimal,
) -> FlexPoolTestHelper {
    let mut helper = helper.unwrap_or(instantiate_helper_with_liquidity());
    helper.swap_success(
        match swap_type {
            SwapType::BuyX => helper.y_address(),
            SwapType::SellX => helper.x_address(),
        },
        input_amount,
        output_amount_expected,
    );
    helper
}

#[test]
fn test_swap_sell_x_minimum_amounts() {
    swap_expect_success(None, SwapType::SellX, Decimal::ZERO, Decimal::ZERO);
    swap_expect_success(None, SwapType::SellX, Decimal::ATTO, Decimal::ZERO);
    swap_expect_success(None, SwapType::SellX, dec!(0.0000000000104), Decimal::ZERO);
    swap_expect_success(
        None,
        SwapType::SellX,
        dec!(0.000000000010400001),
        dec!(0.00000000000002),
    );
}

#[test]
fn test_swap_buy_x_minimum_amounts() {
    swap_expect_success(None, SwapType::BuyX, Decimal::ZERO, Decimal::ZERO);
    swap_expect_success(None, SwapType::BuyX, Decimal::ATTO, Decimal::ZERO);
    swap_expect_success(None, SwapType::BuyX, dec!(0.000000000002), Decimal::ZERO);
    swap_expect_success(
        None,
        SwapType::BuyX,
        dec!(0.000000000002001),
        dec!(0.0000000000004),
    );
}

#[test]
fn swap_with_min_x_min_y_liquidity_sell_x_min_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::SellX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_min_y_liquidity_sell_x_max_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_expect_success(
        Some(helper),
        SwapType::SellX,
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ZERO,
    );
}

#[test]
fn swap_with_min_x_min_y_liquidity_buy_x_min_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::BuyX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_min_y_liquidity_buy_x_max_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        Decimal::ATTO,
        Decimal::ATTO,
        dec!(0),
        dec!(0),
    );
    // use the rest of Y not in the pool already
    swap_expect_success(
        Some(helper),
        SwapType::BuyX,
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ZERO,
    );
}

#[test]
fn swap_with_min_x_max_y_liquidity_sell_x_min_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(
        Some(helper),
        SwapType::SellX,
        Decimal::ATTO,
        dec!(5352178847647348983069495303.05165433812091206),
    );
}

#[test]
fn swap_with_min_x_max_y_liquidity_sell_x_max_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_expect_success(
        Some(helper),
        SwapType::SellX,
        MAX_SUPPLY - Decimal::ATTO,
        dec!(5708990770823838953334066795.414028122216598716),
    );
}

#[test]
fn swap_with_min_x_max_y_liquidity_buy_x_min_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY - Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::BuyX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_min_x_max_y_liquidity_buy_x_max_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        Decimal::ATTO,
        MAX_SUPPLY - Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::BuyX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_sell_x_min_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::SellX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_sell_x_max_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::SellX, Decimal::ATTO, dec!(0));
}

#[test]
fn swap_with_max_x_min_y_liquidity_buy_x_min_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(
        Some(helper),
        SwapType::BuyX,
        Decimal::ATTO,
        dec!(908320896921531716460322305.998358662831085419),
    );
}

#[test]
fn swap_with_max_x_min_y_liquidity_buy_x_max_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY,
        Decimal::ATTO,
        dec!("75557.863725914323419136"),
        dec!(0),
        dec!(0),
    );
    swap_expect_success(
        Some(helper),
        SwapType::BuyX,
        MAX_SUPPLY - Decimal::ATTO,
        dec!(5708810236684072121027370423.896945574596729909),
    );
}

#[test]
fn swap_with_max_x_max_y_liquidity_sell_x_min_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    swap_expect_success(Some(helper), SwapType::SellX, Decimal::ATTO, Decimal::ZERO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_sell_x_max_x() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // use the rest of X not in the pool already
    swap_expect_success(Some(helper), SwapType::SellX, Decimal::ATTO, Decimal::ZERO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_buy_x_min_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY,
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // expect one attos of X
    swap_expect_success(Some(helper), SwapType::BuyX, Decimal::ATTO, Decimal::ZERO);
}

#[test]
fn swap_with_max_x_max_y_liquidity_buy_x_max_y() {
    let mut helper = instantiate_helper();
    helper.add_liquidity_success(
        MAX_SUPPLY,
        MAX_SUPPLY - Decimal::ATTO,
        MAX_SUPPLY,
        dec!(0),
        dec!(0),
    );
    // use the rest of Y not in the pool already, expect one attos of X
    swap_expect_success(Some(helper), SwapType::BuyX, Decimal::ATTO, Decimal::ZERO);
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
    helper.instantiate_default_with_all_fees(dec!(0.1), dec!(0), dec!(0.8), false);
    helper.add_liquidity_success(dec!(10), dec!(250), dec!(50), dec!(0), dec!(0));
    // take output amount
    helper.swap_success(XRD, dec!(10), dec!(0.088028));
    // take protocol fees from input
    helper.swap_success(stable_address, dec!(1.929392), dec!(123.6845968744525425));
}

#[test]
fn swap_multiple() {
    let mut helper: FlexPoolTestHelper =
        swap_expect_success(None, SwapType::SellX, dec!(10), dec!("0.49996875156192"));
    helper.swap_success(helper.x_address(), dec!(1), dec!("0.049993438016644823"));
}
