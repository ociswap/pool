use std::cmp::min;

use crate::constants::POW_ERROR_MARGIN;
use common::{
    math::{AttoPreciseDecimal, DivisibilityRounding},
    pools::SwapType,
};
use scrypto::prelude::*;
use scrypto_math::PowerDecimal;

/// Trait to check if a Decimal value is between zero and one.
pub trait DecimalPercentage {
    /// Returns true if the Decimal value is between zero and one, inclusive.
    fn between_zero_and_one(&self) -> bool;
}

impl DecimalPercentage for Decimal {
    fn between_zero_and_one(&self) -> bool {
        &Self::ZERO <= self && self <= &Self::ONE
    }
}

/// Calculates the net input amount after applying fees.
///
/// This function computes the net input amount by deducting the total fee from the input amount.
/// It also splits the total fee into protocol and liquidity provider shares.
///
/// # Arguments
///
/// * `input_amount` - The initial amount of input tokens.
/// * `input_fee_rate` - The fee rate applied to the input amount. Must be between 0 and 1.
/// * `fee_protocol_share` - The share of the total fee that goes to the protocol. Must be between 0 and 1.
/// * `divisibility` - The number of decimal places to which the amounts should be rounded.
///
/// # Returns
///
/// * `(Decimal, Decimal, Decimal)` - A tuple containing:
///   - The net input amount after fees.
///   - The fee amount allocated to the liquidity provider.
///   - The fee amount allocated to the protocol.
///
/// # Panics
///
/// This function will panic if:
/// * `input_fee_rate` is not between 0 and 1.
/// * The calculated net input amount is negative.
pub fn input_amount_net(
    input_amount: Decimal,
    input_fee_rate: Decimal,
    fee_protocol_share: Decimal,
    divisibility: u8,
) -> (Decimal, Decimal, Decimal) {
    assert!(
        input_fee_rate.between_zero_and_one(),
        "Input fee rate must be between zero and one!"
    );

    // Convert input amount to PreciseDecimal for higher precision calculations
    let input_amount_gross: PreciseDecimal = input_amount.into();
    /*
    Valid pre-conditions:
      `0 <= input_fee_rate <= 1`
      => `0 <= input_amount_gross * input_fee_rate <= input_amount_gross`
      => ceiling to the 18th decimal can lead to `input_fee_lp > input_amount_gross` (with input_fee_rate = 1)
         but only if input_amount_gross has non-zero digits afte the 18th decimal place
         otherwise it is guaranteed that `input_fee_lp <= input_amount_gross`
      => since input_amount_gross is converted from Decimal (with only 18 decimal places) it is strictly true that:
         `input_fee_lp < input_amount_gross`
    Therefore:
      input_amount_net >= 0
    In other words the calculated input_amount_net is always positve or equal.
    */

    // Calculate the total fee by applying the fee rate and rounding up to the specified divisibility
    let input_fee_total: Decimal = (input_amount_gross * input_fee_rate).ceil_to(divisibility);

    // Calculate the protocol fee by applying the protocol share and rounding down
    let input_fee_protocol = (input_fee_total * fee_protocol_share).floor_to(divisibility);

    // Calculate the liquidity provider fee as the remainder of the total fee
    let input_fee_lp: Decimal = input_fee_total - input_fee_protocol;

    let input_amount_net: Decimal = input_amount - input_fee_total;

    assert!(
        input_amount_net >= Decimal::ZERO,
        "Input amount net needs to be positive or zero!"
    );

    (input_amount_net, input_fee_lp, input_fee_protocol)
}

/// Calculates the output amount for a swap operation.
///
/// This function determines the amount of output tokens received from a swap operation,
/// based on the input vault, output vault, net input amount, ratio, swap type, and token divisibility.
///
/// # Arguments
///
/// * `input_vault` - The amount of tokens in the input vault.
/// * `output_vault` - The amount of tokens in the output vault.
/// * `input_amount_net` - The net amount of input tokens after fees.
/// * `ratio` - The ratio of the token weights (x_share / y_share).
/// * `swap_type` - The type of swap, either buying or selling the input token.
/// * `divisibility` - The number of decimal places to which the output amount should be rounded.
///
/// # Returns
///
/// * `Decimal` - The calculated output amount, rounded to the specified divisibility.
///
/// # Panics
///
/// This function will panic if `input_amount_net` is negative or if the calculated `output_amount` is negative.
pub fn output_amount(
    input_vault: Decimal,
    output_vault: Decimal,
    input_amount_net: Decimal,
    ratio: Decimal,
    swap_type: SwapType,
    divisibility: u8,
) -> Decimal {
    // Ensure the net input amount is non-negative
    assert!(
        input_amount_net >= Decimal::ZERO,
        "Input amount net needs to be positive or zero!"
    );

    // Convert input and output vault amounts to PreciseDecimal for higher precision calculations
    let input_vault: PreciseDecimal = input_vault.into();
    let output_vault: PreciseDecimal = output_vault.into();

    // Calculate the output amount based on whether the pool is balanced or imbalanced
    let output_amount = (if ratio == Decimal::ONE {
        // Balanced pool calculation
        output_amount_balanced(input_vault, output_vault, input_amount_net)
    } else {
        // Imbalanced pool calculation
        output_amount_imbalanced(
            input_vault,
            output_vault,
            input_amount_net,
            ratio,
            swap_type,
        )
    })
    .floor_to(divisibility);

    // Ensure the calculated output amount is non-negative
    assert!(
        output_amount >= Decimal::ZERO,
        "Output amount needs to be positive or zero!"
    );

    output_amount
}

/// Calculates the output amount for a balanced pool swap.
///
/// This function uses the constant product formula to ensure the pool invariant `k = X * Y` remains constant.
/// It determines the output amount based on the input amount and the current reserves in the pool.
///
/// # Arguments
///
/// * `input_vault` - The amount of tokens in the input vault.
/// * `output_vault` - The amount of tokens in the output vault.
/// * `input_amount_net` - The net amount of input tokens.
///
/// # Returns
///
/// * `PreciseDecimal` - The calculated output amount.
///
/// # Details
///
/// The function follows these steps:
/// 1. Ensures the pool invariant `k = X * Y` remains constant.
/// 2. Uses the formula `out = (R_o * in) / (R_i + in)` to calculate the output amount.
/// 3. Ensures that the output amount is never more than allowed by the pool invariant.
///
/// # Panics
///
/// This calculation can panic in case of a math overflow, which is the intended safe behavior for the swap of tokens.
fn output_amount_balanced(
    input_vault: PreciseDecimal,
    output_vault: PreciseDecimal,
    input_amount_net: Decimal,
) -> PreciseDecimal {
    /*
    Pool invariant (with pool reserves X and Y):
      k = X * Y
    Mathematical derivation (with input pool reserve R_i and output pool reserve R_o):
      To calculate the output `out` based on an input `in` we use the pool variant `k = X * Y` ensuring that k is constant:
      k = R_i * R_o
      k = (R_i + in) * (R_o - out)
      =>  R_i * R_o = (R_i + in) * (R_o - out)
          R_o - out = (R_i * R_o) / (R_i + in)
          out = R_o - (R_i * R_o) / (R_i + in)
              = R_o * (R_i + in) / (R_i + in) - (R_i * R_o) / (R_i + in)
              = (R_o * (R_i + in) - R_i * R_o) / (R_i + in)
              = (R_o * R_i + R_o * in - R_i * R_o) / (R_i + in)
              = (R_o * in) / (R_i + in)
    Valid pre-conditions:
      - input_vault, output_vault and input_amount_net are exact values
      - `input_vault + input_amount_net` is also an exact result
      - `input_amount_net * output_vault` is smaller than the exact result (due to integer multiplication)
      - the result of integer division is also slightly smaller than the exact result
    Therefore:
      output_amount = `slightly smaller value / exact value` <= `exact output value`
    In other words we are never taking more out of the pool than is allowed by the pool invariant `k = X * Y`.
    */

    // This calculation can panic in case of an math overflow which is the indented safe behaviour for the swap of tokens
    let output_amount = (output_vault * input_amount_net) / (input_vault + input_amount_net);

    output_amount
}

/// Calculates the output amount for an imbalanced pool swap.
///
/// This function is based on the Balancer protocol. It determines the output amount
/// when swapping tokens in an imbalanced pool, where the pool's token weights are not equal.
///
/// # Arguments
///
/// * `input_vault` - The amount of tokens in the input vault.
/// * `output_vault` - The amount of tokens in the output vault.
/// * `input_amount_net` - The net amount of input tokens.
/// * `ratio` - The ratio of the token weights (x_share / y_share).
/// * `swap_type` - The type of swap, either buying or selling the input token.
///
/// # Returns
///
/// * `PreciseDecimal` - The calculated output amount.
///
/// # Details
///
/// The function first calculates the weight based on the swap type:
/// - For `SwapType::BuyX`, the weight is the inverse of the ratio.
/// - For `SwapType::SellX`, the weight is the ratio itself.
///
/// It then computes the `output_vault_share`, which represents the share of the output vault
/// after the swap. This share is adjusted to handle potential precision errors in the power calculation.
///
/// Finally, the function calculates the `output_amount` by multiplying the `output_vault` by the
/// difference between 1 and the `output_vault_share`. This ensures that a larger `output_vault_share`
/// results in a smaller `output_amount`.
fn output_amount_imbalanced(
    input_vault: PreciseDecimal,
    output_vault: PreciseDecimal,
    input_amount_net: Decimal,
    ratio: Decimal,
    swap_type: SwapType,
) -> PreciseDecimal {
    // Calculate the input vault's share after the swap by dividing input vault amount by total amount.
    // We add ATTO to ensure the share is slightly larger than exact value, which favors the pool
    // by effectively reducing the input amount used in calculations.
    let input_vault_share = ((input_vault / (input_vault + input_amount_net))
        + PreciseDecimal::ATTO)
        .checked_truncate(RoundingMode::ToPositiveInfinity)
        .unwrap();

    // Determine the weight based on the swap type
    // ratio = x_share / y_share
    // The division truncating the inverse ratio leads to a potentially larger output share,
    // with input_vault_share in [0, 1] and weight in [0.05, 20], which is safe for the pool.
    let weight = match swap_type {
        SwapType::BuyX => dec!(1) / ratio, // the weight is the inverse of the ratio
        SwapType::SellX => ratio,          // the weight is the ratio itself
    };

    // Calculate the share of the output vault after the swap.
    // Output vault share needs to be larger to ensure safe handling by the pool.
    // We add a small correction value to account for precision errors in pow().
    // When input_vault_share is small and weight is large, output_vault_share
    // could be very small or zero. The safety margin ensures it stays within bounds.
    let output_vault_share = input_vault_share.pow(weight).unwrap();
    let output_vault_share = min(dec!(1), output_vault_share + POW_ERROR_MARGIN); // Adjust for precision errors

    // Calculate the output amount by multiplying the output vault by the complement of the output vault share.
    // A larger output_vault_share means less tokens are available for output.
    let output_amount = output_vault * (dec!(1) - output_vault_share);

    output_amount
}

/// Calculates the square root of the price based on the given amounts and ratio.
///
/// # Arguments
///
/// * `x_amount` - The amount of token X.
/// * `y_amount` - The amount of token Y.
/// * `ratio` - The ratio of the token weights (x_share / y_share).
///
/// # Returns
///
/// * `Option<PreciseDecimal>` - The calculated square root of the price, or `None` if inputs are invalid.
pub fn price_sqrt(x_amount: Decimal, y_amount: Decimal, ratio: Decimal) -> Option<PreciseDecimal> {
    // Ensure the amounts are positive before proceeding with the calculation
    if x_amount <= Decimal::ZERO || y_amount <= Decimal::ZERO {
        return None;
    }

    // The calculation involves:
    // 1. Taking the square root of `y_amount`.
    // 2. Dividing it by the square root of `x_amount`.
    // 3. Multiplying the result by the square root of the `ratio`.
    //
    // Final result is the square root of the price, adjusted by the ratio.
    PreciseDecimal::from(y_amount)
        .checked_sqrt()?
        .checked_div(PreciseDecimal::from(x_amount).checked_sqrt()?)?
        .checked_mul(ratio.checked_sqrt()?)
}
