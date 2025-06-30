# BVS Vault CW20 Tokenized

The BVS Vault CW20 Tokenized is a specialized vault contract
designed to support CW20 tokens as the underlying asset in the SatLayer ecosystem.
It extends the functionality of the standard BVS Vault CW20 by implementing a tokenized vault
that issues receipt tokens (CW20 tokens) to represent shares in the vault.

The Vault CW20 Tokenized implements the ERC-4626 tokenized vault standard,
providing a standardized interface for yield-bearing vaults.
It manages the relationship between deposited CW20 tokens and receipt tokens (shares),
allowing for efficient tracking of user deposits and their corresponding value.

- **CW20 Token Support**: Handles CW20 tokens as the underlying asset
- **Tokenized Shares**: Issues receipt tokens (CW20 tokens) to represent shares in the vault
- **Share-Based Accounting**: Uses a share system to track user deposits and their value
- **CW20 Compliant Receipt Tokens**: Receipt tokens are themselves CW20 tokens that can be transferred, sent, etc.

## Contract Functions

### Execute Functions

- `DepositFor`: Deposit CW20 tokens into the vault for a recipient, who receives receipt tokens
- `QueueWithdrawalTo`: Queue a withdrawal request for later processing
- `RedeemWithdrawalTo`: Process a queued withdrawal request
- `Transfer`, `Send`, etc.: Standard CW20 operations for the receipt tokens

### Query Functions

- `Shares`: Get the number of shares (receipt tokens) owned by a staker
- `Assets`: Get the asset value of shares owned by a staker
- `ConvertToAssets`: Convert a share amount to its asset value
- `ConvertToShares`: Convert an asset amount to its share value
- `TotalShares`: Get the total number of shares (receipt tokens) in the vault
- `TotalAssets`: Get the total asset value in the vault
- `QueuedWithdrawal`: Get information about a queued withdrawal
- `VaultInfo`: Get general information about the vault
- Standard CW20 queries for the receipt tokens

## Deposit and Withdrawal Process

The Vault CW20 Tokenized manages deposits and withdrawals through a tokenized share-based system:

1. When users deposit CW20 tokens, they receive receipt tokens (CW20 tokens) proportional to their deposit value
2. The receipt tokens represent shares in the vault and can be transferred like any other CW20 token
3. The share price (exchange rate) is determined by the total assets and total shares in the vault
4. When users withdraw, their receipt tokens are burned and they receive CW20 tokens based on the current exchange rate
5. If the operator is not actively validating, withdrawals are queued for later processing

The vault implements a withdrawal queue to handle situations where immediate withdrawals are not possible:

1. Users request a withdrawal via `QueueWithdrawalTo`
2. The request is stored with a timestamp and the current exchange rate
3. After the lock period expires, users can redeem their withdrawal via `RedeemWithdrawalTo`
4. The withdrawal is processed using the exchange rate from when it was queued

## Integration with Other BVS Contracts

BVS Vault CW20 Tokenized integrates with the following BVS contracts:

- `bvs-pauser`: Contract for pausing functionality
- `bvs-registry`: Contract for registry functionality
- `bvs-vault-router`: Contract for routing between different vaults
- `bvs-vault-base`: Base functionality for all vault contracts
- `bvs-vault-cw20`: Underlying functionality for CW20 vaults
