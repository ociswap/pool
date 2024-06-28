use common::math::AttoDecimal;
// INSTANTIATE
use flex_pool_test_helper::*;
use pretty_assertions::assert_eq;
use scrypto::prelude::*;
use scrypto_testenv::*;
use test_case::test_case;

#[test]
fn test_instantiate() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    let receipt = helper
        .instantiate(
            helper.x_address(),
            helper.y_address(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_success(false);
    let outputs: Vec<(ComponentAddress, ResourceAddress)> = receipt.outputs("instantiate");
    let commit_result = receipt.execution_receipt.expect_commit_success();
    assert_eq!(outputs[0].0, commit_result.new_component_addresses()[0]);
}

#[test]
fn test_instantiate_same_token() {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.x_address(),
            helper.x_address(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_instantiate_random_address_token() {
    let mut helper = FlexPoolTestHelper::new();
    println!("{:?}", helper.a_address());
    println!("{:?}", helper.b_address());
    // random hex string with 5d as first two chars
    let random_address = ResourceAddress::try_from_hex(
        "5df173925d7814e488512f12cb03c6edfe2b3ea39c24538290476c34ba17",
    )
    .unwrap();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.x_address(),
            random_address,
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_rejection(false);
}

#[test]
fn test_instantiate_nft_addresses_both() {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.j_nft_address(),
            helper.k_nft_address(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_instantiate_nft_address_x() {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.j_nft_address(),
            helper.y_address(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_instantiate_nft_address_y() {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.x_address(),
            helper.k_nft_address(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_instantiate_pool_with_lp_token() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper
        .instantiate(
            helper.x_address(),
            helper.lp_address.unwrap(),
            dec!(0),
            dec!(0.5),
            helper.registry.registry_address.unwrap(),
        )
        .registry
        .execute_expect_success(false); // We can have a Pool with lp tokens.
}

#[test]
fn test_instantiate_wrong_order() {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate(
        helper.y_address(),
        helper.x_address(),
        dec!(0),
        dec!(0.5),
        helper.registry.registry_address.unwrap(),
    );
    let receipt = helper.registry.execute_expect_success(false);
    let (pool_address, _): (ComponentAddress, ResourceAddress) = receipt.outputs("instantiate")[0];
    helper.pool_address = Some(pool_address);
    helper
        .add_liquidity(helper.x_address(), dec!(1), helper.y_address(), dec!(1))
        .registry
        .execute_expect_success(false);
}

#[test_case(-Decimal::ATTO, false ; "negative")]
#[test_case(dec!("0"), true ; "zero")]
#[test_case(Decimal::ATTO, true ; "atto")]
#[test_case(dec!("0.05"), true ; "valid")]
#[test_case(dec!("1"), false ; "one")]
#[test_case(dec!(1) + Decimal::ATTO, false ; "more_than_one")]
fn test_instantiate_input_fee_rate(input_fee_rate: Decimal, success: bool) {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate(
        helper.x_address(),
        helper.y_address(),
        input_fee_rate,
        dec!(0.5),
        helper.registry.registry_address.unwrap(),
    );

    if success {
        helper.registry.execute_expect_success(false);
    } else {
        helper.registry.execute_expect_failure(false);
    }
}

#[test_case(-Decimal::ATTO, false ; "negative")]
#[test_case(dec!("0"), false ; "zero")]
#[test_case(Decimal::ATTO, true ; "atto")]
#[test_case(dec!("0.2"), true ; "valid_1")]
#[test_case(dec!("0.8"), true ; "valid_2")]
#[test_case(dec!("1"), false ; "one")]
#[test_case(dec!(1) + Decimal::ATTO, false ; "more_than_one")]
fn test_instantiate_a_share(a_share: Decimal, success: bool) {
    let mut helper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate(
        helper.x_address(),
        helper.y_address(),
        dec!(0),
        a_share,
        helper.registry.registry_address.unwrap(),
    );

    if success {
        helper.registry.execute_expect_success(false);
    } else {
        helper.registry.execute_expect_failure(false);
    }
}

#[test]
fn test_instantiate_metadata() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(false);
    helper.display_meta("name");
    helper.display_meta("description");
    helper.display_meta("tags");
    helper.display_meta("icon_url");
    helper.display_meta("info_url");
    helper.display_meta("dapp_definition");
    let liquidity_pool_meta = helper
        .registry
        .env
        .test_runner
        .get_metadata(
            helper.liquidity_pool_address.unwrap().into(),
            "dapp_definition",
        )
        .unwrap();
    println!("{:?}", liquidity_pool_meta);
}
