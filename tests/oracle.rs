use ociswap_pool_test_helper::*;
use scrypto::prelude::*;
use scrypto_testenv::TestHelperExecution;

#[test]
fn test_oracle_last_observation_index() {
    let mut helper = PoolTestHelper::new();
    helper.instantiate_default(false);
    helper.add_liquidity_success(
        dec!(100000),
        dec!(5000),
        dec!("22360.679774997896964092"),
        dec!(0),
        dec!(0),
    );
    helper.jump_to_timestamp_minutes(15);
    helper.swap_success(
        helper.x_address(),
        Decimal::ONE,
        dec!("0.04999950000499995"),
    );
    helper.jump_to_timestamp_minutes(30);
    helper.swap_success(
        helper.y_address(),
        Decimal::ONE,
        dec!("19.996400681870824471"),
    );
    helper.jump_to_timestamp_minutes(45);
    helper.swap_success(
        helper.x_address(),
        Decimal::ONE,
        dec!("0.050018501534951247"),
    );
    helper.jump_to_timestamp_minutes(60);

    let receipt = helper
        .last_observation_index()
        .registry
        .execute_expect_success(false);
    let outputs: Vec<Option<u16>> = receipt.outputs("last_observation_index");
    println!("Last observation index: {:?}", outputs);

    assert_eq!(outputs, vec![Some(1)]);

    let receipt = helper
        .oldest_observation_at()
        .registry
        .execute_expect_success(false);
    let outputs: Vec<Option<u64>> = receipt.outputs("oldest_observation_at");

    assert_eq!(outputs, vec![Some(1800)]);
}
