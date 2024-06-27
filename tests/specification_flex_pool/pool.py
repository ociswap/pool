from decimal import Decimal
import logging
from typing import Tuple
from scryptomath import Decimal as SDecimal, PreciseDecimal


class Pool:
    def __init__(
        self,
        fee_input_rate: Decimal = None,
        fee_output_rate: Decimal = None,
        fee_protocol_share: Decimal = None,
    ) -> None:
        self.fee_input_rate = fee_input_rate or Decimal(0)
        self.fee_output_rate = fee_output_rate or Decimal(0)
        self.fee_protocol_share = fee_protocol_share or Decimal(0)

        self.lp_total = Decimal(0)
        self.x_vault = Decimal(0)
        self.y_vault = Decimal(0)

        self.x_fee_protocol = Decimal(0)
        self.y_fee_protocol = Decimal(0)

        print(
            f"[INSTANTIATE] fee_input_rate={self.fee_input_rate:f}, fee_output_rate={self.fee_output_rate:f}, fee_protocol_share={self.fee_protocol_share:f}"
        )

    def _calculate_target_add_liquidity(
        self, x_amount: Decimal, y_amount: Decimal
    ) -> Tuple[int, int]:
        price = self.y_vault / self.x_vault
        new_price = y_amount / x_amount

        x_add, y_add = (
            (x_amount, x_amount * price)
            if price < new_price
            else (y_amount / price, y_amount)
        )

        return x_add, y_add

    def add_liquidity(
        self, x_amount: Decimal, y_amount: Decimal
    ) -> (Decimal, Decimal, Decimal):
        if self.lp_total == Decimal(0):
            lp_minted = (x_amount * y_amount).sqrt()
            lp_minted = Decimal(
                SDecimal(lp_minted) + Decimal("0.000000000000000001")
            )  # adapt behaviour to native pool
            self.lp_total += lp_minted
            self.x_vault += x_amount
            self.y_vault += y_amount
            return (lp_minted, Decimal(0), Decimal(0))

        x_add, y_add = self._calculate_target_add_liquidity(x_amount, y_amount)

        liquidity_amount = min(
            x_add / self.x_vault * self.lp_total,
            y_add / self.y_vault * self.lp_total,
        )

        self.x_vault += x_add
        self.y_vault += y_add
        self.lp_total += liquidity_amount

        x_remainder = x_amount - x_add
        y_remainder = y_amount - y_add

        print(
            f"[ADD LIQUIDITY] x_added={SDecimal(x_add):f}, y_added={SDecimal(x_add):f}, x_remainder={SDecimal(x_remainder):f}, y_remainder={SDecimal(y_remainder):f}"
        )

        return liquidity_amount, x_remainder, y_remainder

    def remove_liquidity(self, lp_amount: Decimal) -> (Decimal, Decimal):
        if self.lp_total < lp_amount:
            raise ValueError(
                "Removing more than the liquidity available is not allowed"
            )
        ratio = lp_amount / self.lp_total
        x_amount = ratio * self.x_vault
        y_amount = ratio * self.y_vault
        self.lp_total -= lp_amount
        self.x_vault -= x_amount
        self.y_vault -= y_amount
        print(
            f"[REMOVE LIQUIDITY] x_removed={SDecimal(x_amount):f}, y_removed={SDecimal(y_amount):f}"
        )
        return x_amount, y_amount

    def _calculate_input_net(self, input_amount: Decimal):
        fee_total = input_amount * self.fee_input_rate
        fee_protocol = fee_total * self.fee_protocol_share
        fee_lp = fee_total - fee_protocol

        input_net = input_amount - fee_lp - fee_protocol
        if input_net < 0:
            raise ValueError(
                "Input too low for minimum fee. Input net amount can not be negative."
            )
        return input_net, fee_lp, fee_protocol

    def swap(self, input_amount: Decimal, input_is_x: bool) -> Decimal:
        input_net, input_fee_lp, input_fee_protocol = self._calculate_input_net(
            input_amount
        )
        output_net, output_fee_lp, output_fee_protocol = self._calculate_output_net(
            input_net, input_is_x
        )

        if input_is_x:
            self.x_fee_protocol += input_fee_protocol
            self.y_fee_protocol += output_fee_protocol
            self.x_vault += input_net + input_fee_lp
            self.y_vault -= output_net - output_fee_protocol
        else:
            self.x_fee_protocol += output_fee_protocol
            self.y_fee_protocol += input_fee_protocol
            self.x_vault -= output_net - output_fee_protocol
            self.y_vault += input_net + input_fee_lp

        print(
            f"[SWAP] output_net={SDecimal(output_net):f}, input_fee_lp={SDecimal(input_fee_lp):f}, input_fee_protocol={SDecimal(input_fee_protocol):f}, output_fee_lp={SDecimal(output_fee_lp):f}, output_fee_protocol={SDecimal(output_fee_protocol):f}"
        )

        return (
            output_net,
            input_fee_lp,
            input_fee_protocol,
            output_fee_lp,
            output_fee_protocol,
        )

    @property
    def price_sqrt(self):
        price_sqrt = PreciseDecimal((self.y_vault / self.x_vault).sqrt())
        print(f"[PRICE_SQRT] price_sqrt={price_sqrt}")
        return price_sqrt

    def _calculate_output_net(self, input_amount_net: int, input_is_x: bool):
        if input_amount_net < 0:
            raise ValueError("Negative input amount is not allowed.")
        input_vault, output_vault = (
            (self.x_vault, self.y_vault) if input_is_x else (self.y_vault, self.x_vault)
        )

        output_amount = output_vault - (input_vault * output_vault) / (
            input_vault + input_amount_net
        )
        if output_amount == output_vault:
            raise ValueError("Output amount is not allowed to take whole pool.")
        fee_total = output_amount * self.fee_output_rate
        fee_protocol = fee_total * self.fee_protocol_share
        fee_lp = fee_total - fee_protocol

        output_amount_net = output_amount - fee_lp - fee_protocol

        return output_amount_net, fee_lp, fee_protocol
