# BVS Vault Bank

The BVS Vault Bank is a specialized vault contract
designed to support native bank tokens as the underlying asset in the SatLayer ecosystem.
It enables operators to manage deposits and withdrawals of native tokens
like `ubbn` (the native token of the Babylon project).

The Vault Bank implements the ERC-4626 tokenized vault standard,
providing a standardized interface for yield-bearing vaults.
It manages the relationship between deposited assets and shares,
allowing for efficient tracking of user deposits and their corresponding value.

- **Native Token Support**: Handles native bank tokens as the underlying asset
- **Share-Based Accounting**: Uses a share system to track user deposits and their value

## Contract Functions

### Execute Functions

- `DepositFor`: Deposit native tokens into the vault for a recipient
- `QueueWithdrawalTo`: Queue a withdrawal request for later processing
- `RedeemWithdrawalTo`: Process a queued withdrawal request

### Query Functions

- `Shares`: Get the number of shares owned by a staker
- `Assets`: Get the asset value of shares owned by a staker
- `ConvertToAssets`: Convert a share amount to its asset value
- `ConvertToShares`: Convert an asset amount to its share value
- `TotalShares`: Get the total number of shares in the vault
- `TotalAssets`: Get the total asset value in the vault
- `QueuedWithdrawal`: Get information about a queued withdrawal
- `VaultInfo`: Get general information about the vault

## Deposit and Withdrawal Process

The Vault Bank manages deposits and withdrawals through a share-based system:

1. When users deposit assets, they receive shares proportional to their deposit value
2. The share price (exchange rate) is determined by the total assets and total shares in the vault
3. When users withdraw, their shares are converted back to assets based on the current exchange rate
4. If the operator is not actively validating, withdrawals are queued for later processing

The vault implements a withdrawal queue to handle situations where immediate withdrawals are not possible:

1. Users request a withdrawal via `QueueWithdrawalTo`
2. The request is stored with a timestamp and the current exchange rate
3. After the lock period expires, users can redeem their withdrawal via `RedeemWithdrawalTo`
4. The withdrawal is processed using the exchange rate from when it was queued
