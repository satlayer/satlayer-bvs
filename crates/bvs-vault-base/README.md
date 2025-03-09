# BVS Vault Base

> [!NOTE]  
> This is not a CosmWasm contract.

This crate provides the base utilities for the BVS Vault contract.

### `bvs_vault_base::offset`

This module provides a virtual offset utility for the BVS Vault contract.
Follows the OpenZeppelin's ERC4626 mitigation strategy for inflation attack.

### `bvs_vault_base::router`

Router functionality for the BVS Vault contract, for authorizing calls, and for calling the router contract.

### `bvs_vault_base::shares`

Accounting functions to keep track of staker shares (non-tokenized accounting).

### `bvs_vault_base::msg`

Provides the `ExecuteMsg` and `QueryMsg` interface for the BVS Vault contract.
Downstream contracts must implement these messages.
