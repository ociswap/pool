use common::math::*;
use ociswap_pool_test_helper::PoolTestHelper;
use pretty_assertions::assert_eq;
use radix_engine::system::system_modules::execution_trace::ResourceSpecifier;
use scrypto::prelude::*;
use scrypto_testenv::environment::TestHelperExecution;

#[test]
fn test_getters_after_instantiation() {
    let input_fee_rate = dec!(0.003251);
    let x_share = dec!(0.615);
    let flash_loan_fee_rate = dec!(0.01021);

    let fee_protocol_share = dec!(0.0187136);

    let mut helper = PoolTestHelper::new_without_instantiate_registry();
    helper.registry.instantiate_execute(
        helper.registry.admin_badge_address(),
        fee_protocol_share,
        1,
        1,
    );
    helper.set_whitelist_registry();
    helper.instantiate_full_direct(
        helper.a_address(),
        helper.b_address(),
        input_fee_rate,
        flash_loan_fee_rate,
        x_share,
        false,
    );

    helper.price_sqrt();
    helper.total_liquidity();
    helper.input_fee_rate();
    helper.fee_protocol_share();
    helper.flash_loan_fee_rate();
    helper.x_share();
    helper.y_share();

    let receipt = helper.registry.execute_expect_success(false);

    let price_sqrt_returned: Vec<Option<PreciseDecimal>> = receipt.outputs("price_sqrt");
    let total_liquidity_returned: Vec<IndexMap<ResourceAddress, Decimal>> =
        receipt.outputs("total_liquidity");
    let input_fee_rate_returned: Vec<Decimal> = receipt.outputs("input_fee_rate");
    let fee_protocol_share_returned: Vec<Decimal> = receipt.outputs("fee_protocol_share");
    let flash_loan_fee_rate_returned: Vec<Decimal> = receipt.outputs("flash_loan_fee_rate");
    let x_share_returned: Vec<Decimal> = receipt.outputs("x_share");
    let y_share_returned: Vec<Decimal> = receipt.outputs("y_share");

    assert_eq!(
        (
            price_sqrt_returned,
            total_liquidity_returned,
            input_fee_rate_returned,
            fee_protocol_share_returned,
            flash_loan_fee_rate_returned,
            x_share_returned,
            y_share_returned
        ),
        (
            vec![None],
            vec![indexmap! { helper.x_address() => dec!(0), helper.y_address() => dec!(0) }],
            vec![input_fee_rate],
            vec![dec!(0)],
            vec![flash_loan_fee_rate],
            vec![x_share],
            vec![dec!(1) - x_share]
        )
    );
}

#[test]
fn test_lp_address() {
    let mut helper = PoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate_default(false);

    helper.add_liquidity_default(dec!(100), dec!(100));
    helper.lp_address();

    let receipt = helper.registry.execute_expect_success(false);

    let output_buckets = receipt.output_buckets("add_liquidity");
    let lp_address: Vec<ResourceAddress> = receipt.outputs("lp_address");

    assert_eq!(output_buckets.len(), 1);
    assert_eq!(output_buckets[0].len(), 1);
    let expected_lp_address =
        if let ResourceSpecifier::Amount(expected_address, _) = output_buckets[0][0] {
            expected_address
        } else {
            panic!("Expected ResourceSpecifier::Amount");
        };

    assert_eq!(lp_address, vec![expected_lp_address]);
}

#[test]
fn test_after_first_transaction() {
    let input_fee_rate = dec!(0.003251);
    let x_share = dec!(0.615);
    let flash_loan_fee_rate = dec!(0.01021);

    let fee_protocol_share = dec!(0.0187136);

    let mut helper = PoolTestHelper::new_without_instantiate_registry();
    helper.registry.instantiate_execute(
        helper.registry.admin_badge_address(),
        fee_protocol_share,
        1,
        1,
    );
    helper.set_whitelist_registry();
    helper.instantiate_full_direct(
        helper.a_address(),
        helper.b_address(),
        input_fee_rate,
        flash_loan_fee_rate,
        x_share,
        false,
    );

    helper.add_liquidity_default(dec!(100), dec!(100));
    helper.total_liquidity();
    helper.lp_total_supply();

    let receipt_1 = helper.registry.execute_expect_success(false);

    helper.swap(helper.y_address(), Decimal::ATTO);
    helper.fee_protocol_share();
    let receipt_2 = helper.registry.execute_expect_success(false);

    let total_liquidity_returned: Vec<IndexMap<ResourceAddress, Decimal>> =
        receipt_1.outputs("total_liquidity");
    let fee_protocol_share_returned: Vec<Decimal> = receipt_2.outputs("fee_protocol_share");

    let lp_total_supply_returned: Vec<Decimal> = receipt_1.outputs("lp_total_supply");

    assert_eq!(
        (
            total_liquidity_returned,
            fee_protocol_share_returned,
            lp_total_supply_returned
        ),
        (
            vec![indexmap! { helper.x_address() => dec!(100), helper.y_address() => dec!(100) }],
            vec![fee_protocol_share],
            vec![dec!(100)]
        )
    );
}
