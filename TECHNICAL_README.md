# Flex Pool Blueprint

## Overview

The Flex Pool blueprint is designed for managing a liquidity pool in a decentralized finance (DeFi) environment. It supports the provisioning of imbalanced liquidity, allowing it to cater to a multitude of use cases. It handles liquidity provision, swaps, fee management, and supports customizability via hooks. It also supports advanced features like flash loans and integrates with a global registry for protocol fee updates, ensuring the pool operates with the most recent protocol configuration.

## Key Components and Features

### FlexPool Component

The `FlexPool` component is the main component aggregating all the functionality and holds the state and logic for the liquidity pool. It contains all the logic necessary for adding/removing liquidity, performing swaps, fee management, hooks management for extended functionalities, and an oracle to access reliable past price data.

### Price Oracle

The Price Oracle present in the Flex Pool (but also in the Precision Pool) brings to the Radix DLT and its users the ability to have access to decentralized and reliable feeds of past price data, crucial to many DeFi applications. Read more about it in the [Oracle documentation](../common/Oracle.md).

### Hooks

Hooks are a powerful feature in the Flex Pool that allow developers to extend and customize the pool's functionality without modifying its core logic. By leveraging hooks, developers can inject custom behavior at various points in the pool's lifecycle, such as immediately before/after adding liquidity, before/after swapping, and before/after removing liquidity. This modular approach enables the integration of additional features like dynamic fees, external data validation, or complex business logic, ensuring that the pool can adapt to a wide range of use cases and evolving requirements without necessitating core changes. Hooks provide a flexible and scalable way to enhance the pool's capabilities while maintaining the integrity and security of the core system.

### Flash Loans

Flash loans are an advanced feature supported by the Flex Pool, allowing users to borrow assets without requiring collateral, provided that the borrowed amount is returned within the same transaction. This capability is particularly useful for arbitrage opportunities, collateral swapping, and other complex financial strategies that require temporary liquidity.

The Flash Loan works via the emission of a special transient token, passed to the user when a loan is taken, along with the loaned tokens themselves. This token is not depositable and therefore must be burned, but only the pool itself has the authority to do so. In order for the user to end the loan, the transient token and the tokens must be passed back to the pool. Upon receiving them, the pool asserts that the required amount was paid back and, if that is the case, it burns the transient token, concluding the loan.

## Pool Mechanisms

### Liquidity Management

The Flex Pool uses a custom version of the Constant Product Market Maker in order to price the tokens. This version allows pools to be set up either with balanced liquidity (e.g., 50/50) or imbalanced liquidity (e.g., 80/20) based on the initial share distribution `a_share` specified during the pool's instantiation. This distribution affects the trading dynamics and the pricing algorithm used within the pool.

The formula for the custom Constant Product Market Maker used is:

\[ x^{s_x} \times y^{s_y} = k \]

where:
- \( x \) and \( y \) are the amounts of the two tokens in the pool.
- \( s_x \) and \( s_y \) are the normalized value shares of the respective tokens.
- \( k \) is a constant. It remains invariant during swaps and only changes as liquidity is added/removed.

Since the shares are normalized, we can further simplify this to:

\[ x^{s_x} \times y^{1 - s_x} = k \]

The Flex Pool leverages the RDX Works provided component `TwoResourcePool` as the core for managing liquidity. Through it, **fungible LP tokens** are issued. The usage of these LP tokens allows not only for greater financial versatility, but also greater integration with the Radix, with LP tokens being directly displayed in the Pool Units section. Two main functions are provided:

- **Add Liquidity**: Liquidity providers can add liquidity by passing the tokens of the two pool's addresses. If the added tokens are not provided in the right ratio (meaning that the value of the tokens in each bucket is not in equilibrium according to the price of the pool), then the maximum depositable amount of each token is calculated and the remainder is returned, along with the user's LP tokens.

- **Remove Liquidity**: Allows for the withdrawal of tokens by calculating the withdrawable amounts based on the LP tokens passed by the user. This also includes the fees earned by the user for providing liquidity.

### Pool Pricing

Due to the custom Constant Product Formula used, the price \( P \) of the pool (price of \( x \) in terms of \( y \)) is dictated by the following relation:

\[ \text{P} = \frac{y}{x} \times \frac{s_x}{1 - s_x} \]

where:
- \( x \) is the amount of token \( X \) in the pool.
- \( y \) is the amount of token \( Y \) in the pool.
- \( s_x \) is the share of token \( x \) in the pool value.

### Swap

The calculation of the swaps differs depending on whether the pool is balanced or imbalanced, i.e., depending on whether the value share of token X in the pool, \( s_x \), is exactly half or not, respectively.

1) **Balanced case**. In this case, the simplified formula below can be used:

\[ \text{out} = \frac{R_o \cdot \text{in}(1 - f)}{R_i + \text{in}(1 - f)} \]

2) **Imbalanced cases**

\[ \text{out} = R_{o} \times \left[1 - \left( \frac{R_{i}}{R_{i} + \text{in}(1 - f)} \right)^{r} \right] \]

\[ r = \begin{cases}
\frac{s_x}{1 - s_x} & \text{if input is } X \text{tokens} \\
\frac{1 - s_x}{s_x} & \text{if input is } Y \text{tokens}
\end{cases} \]

where:
- \( \text{in} \): The amount of input tokens provided for the swap
- \( \text{out} \): The amount of output tokens received after the swap
- \( R_i \): The reserve of the input token in the pool
- \( R_o \): The reserve of the output token in the pool
- \( f \): The input fee rate
- \( s_x \): The value share of token X in the pool

## Conclusion

The Flex Pool module is designed to provide a robust framework for DeFi applications, enabling efficient liquidity management, fee collection, and token swapping with an emphasis on security, flexibility, and performance. The integration of hooks and oracles further enhances its capabilities, making it adaptable to a wide range of financial operations in the DeFi ecosystem.
