# BVS Vault Router

The BVS Vault Router is a central contract
that manages the interaction between vaults and other contracts in the SatLayer ecosystem.
It serves as the entry/exit point for all vaults,
controlling that vaults can accept deposits and managing the relationship between operators and their vaults.

The Vault Router provides essential functionality for the SatLayer ecosystem by:

1. Managing the whitelist status of vaults, determining which vaults can accept deposits
2. Tracking the relationship between operators and their vaults
3. Querying the registry to determine if operators are validating services
4. Managing withdrawal lock periods for vaults

## Contract Functions

### Execute Functions

- `SetVault`: Set a vault contract and its whitelist status (only owner can call)
- `SetWithdrawalLockPeriod`: Set the lock period for withdrawals (only owner can call)
- `TransferOwnership`: Transfer ownership of the contract to a new owner

### Query Functions

- `IsWhitelisted`: Check if a vault is whitelisted and can accept deposits
- `IsValidating`: Check if an operator is validating services
- `ListVaults`: List all vaults with pagination support
- `ListVaultsByOperator`: List vaults managed by a specific operator
- `WithdrawalLockPeriod`: Get the current withdrawal lock period

## Whitelisting Process

The Vault Router manages vaults through a whitelisting process:

1. The owner adds a vault to the router using `SetVault` with `whitelisted: true`
2. The router verifies that the vault is connected to it
3. The router creates a mapping between the operator and the vault
4. Whitelisted vaults can accept deposits
5. The router can check if an operator is validating services by querying the registry
