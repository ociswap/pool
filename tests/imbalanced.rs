use flex_pool_test_helper::*;
use scrypto::prelude::*;

#[test]
fn test_swap_sell_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), true);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.x_address(), dec!(10), dec!("0.499950004999500049"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), true);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.x_address(), dec!(10), dec!("0.49996875156192"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), true);
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
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), true);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.y_address(), dec!(10), dec!("199.600798403193612774"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), true);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.swap_success(helper.y_address(), dec!(10), dec!("199.7503743916192"));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), true);
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
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.5), true);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127623")));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.8), true);
    helper.add_liquidity_success(
        dec!(400000),
        dec!(5000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127622")));
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default_with_all_fees(dec!(0), dec!(0), dec!(0.2), true);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(20000),
        dec!("44721.359549995793928184"),
        dec!(0),
        dec!(0),
    );
    helper.price_sqrt_success(Some(pdec!("0.223606797749978969640917366873127623")));
}
