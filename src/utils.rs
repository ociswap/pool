use crate::constants::*;
use common::utils::assert_fee_rate_within_bounds;
use scrypto::prelude::*;

pub fn assert_input_fee_rate_is_valid(input_fee_rate: Decimal) {
    assert_fee_rate_within_bounds(input_fee_rate, INPUT_FEE_RATE_MAX, "input fee rate");
}

pub fn assert_flash_loan_fee_rate_is_valid(flash_loan_fee_rate: Decimal) {
    assert_fee_rate_within_bounds(
        flash_loan_fee_rate,
        FLASH_LOAN_FEE_RATE_MAX,
        "flash loan fee rate",
    );
}

pub fn assert_hooks_bucket_output_and_address(
    initial_amount: Decimal,
    initial_address: ResourceAddress,
    returned_tokens: &Bucket,
    hook_type_name: &str,
) {
    assert!(
        initial_amount * HOOKS_MIN_REMAINING_BUCKET_FRACTION <= returned_tokens.amount(),
        "{} hooks took more tokens than the allowed limit of 10%",
        hook_type_name
    );
    assert_eq!(
        initial_address,
        returned_tokens.resource_address(),
        "{} hooks returned a different token than expected",
        hook_type_name
    );
}
