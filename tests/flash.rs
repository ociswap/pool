use scrypto::prelude::*;
use scrypto_testenv::*;
use flex_pool_test_helper::*;

// Flash loan

#[test]
fn test_take_loan_only() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper
        .add_liquidity_default(dec!(10), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .flash_loan(helper.x_address(), dec!(1))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_take_too_much_loan() {
    let mut helper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper
        .add_liquidity_default(dec!(10), dec!(10))
        .registry
        .execute_expect_success(false);
    helper
        .flash_loan(helper.x_address(), dec!(20))
        .registry
        .execute_expect_failure(false);
}

#[test]
fn test_take_repay_loan() {
    let mut helper = FlexPoolTestHelper::new();
    helper.repay_loan_success(helper.x_address(), dec!(1), dec!("0.009"), dec!(0));
}

#[test]
fn test_take_loan_repay_insufficient() {
    let mut helper = FlexPoolTestHelper::new();
    helper.repay_loan_failure(helper.x_address(), dec!(1), dec!(0));
}

#[test]
fn test_take_loan_repay_more() {
    let mut helper = FlexPoolTestHelper::new();
    helper.repay_loan_success(helper.x_address(), dec!(1), dec!("1.009"), dec!(1));
}

#[test]
fn test_take_loan_repay_wrong_token() {
    let mut helper = FlexPoolTestHelper::new();
    helper.repay_loan_failure(helper.y_address(), dec!("1.009"), dec!(0));
}

#[test]
fn test_take_two_loans_one_repay() {
    let mut helper = FlexPoolTestHelper::new();

    helper.instantiate_default(true);
    let receipt = helper
        .add_liquidity_default(dec!(10), dec!(10))
        .flash_loan_address()
        .registry
        .execute_expect_success(false);
    let flash_loan_address: ResourceAddress = receipt.outputs("flash_loan_address")[0];

    helper.flash_loan(helper.x_address(), dec!(1));
    helper.flash_loan(helper.x_address(), dec!(1));
    helper
        .repay_loan(
            helper.x_address(),
            dec!("1"),
            dec!("0.009"),
            flash_loan_address,
            dec!(2),
        )
        .registry
        .execute_expect_failure(false);
}
