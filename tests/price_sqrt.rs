use flex_pool_test_helper::*;
use scrypto::prelude::*;

#[test]
fn test_price_sqrt_no_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.price_sqrt_success(None);
}

#[test]
fn test_price_sqrt_swap() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper.add_liquidity_default(dec!(1), dec!(2));
    helper.price_sqrt_success(Some(pdec!("1.414213562373095048801688724209698078")));
    helper.swap_success(helper.x_address(), dec!(2), dec!("1.333333333333333333"));
    helper.price_sqrt_success(Some(pdec!("0.471404520791031683051747371600990613")));
}

#[test]
fn test_price_sqrt_multiple_add_liquidity() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper.add_liquidity_default(dec!(1), dec!(2));
    helper.price_sqrt_success(Some(pdec!("1.414213562373095048801688724209698078")));
    helper.add_liquidity_default(dec!(1), dec!(2));
    helper.price_sqrt_success(Some(pdec!("1.414213562373095048801688724209698079")));
}

#[test]
fn test_price_sqrt_add_liquidity_wrong_order() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity(helper.y_address(), dec!(2), helper.x_address(), dec!(1));
    helper.price_sqrt_success(Some(pdec!("1.414213562373095048801688724209698078")));
}
