use flex_pool_test_helper::*;
use scrypto::prelude::*;

#[test]
fn test_instantiate_with_liquidity() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate_with_liquidity_success(
        dec!(10),
        dec!(40),
        dec!("0.1"),
        dec!(20),
        dec!(0),
        dec!(0),
    );
}
