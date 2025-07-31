# @satlayer/testcontainers

A package providing testcontainer testing environments for both Ethereum (EVM) and CosmWasm blockchain applications.
This package also includes utilities for bootstrapping SatLayer core contracts multiple control planes (EVM and CosmWasm) for testing purposes.

## Overview

This package simplifies blockchain testing by providing testcontainer abstractions for:

- Ethereum testing via Anvil nodes (forked mainnet or testnet)
- CosmWasm testing via wasmd containers
- Bootstrapping SatLayer core contracts for both EVM and CosmWasm environments

It allows developers to easily set up isolated blockchain environments, deploy contracts, and test complex interactions without affecting external networks.

## Installation

```bash
pnpm install -D @satlayer/testcontainers
```

## Features

### Ethereum (EVM) Node

`AnvilContainer` provides a lightweight Ethereum node for testing purposes. It supports:

- Create isolated Anvil containers for Ethereum testing
- Deploy and interact with EVM contracts
- Create test accounts and fund them
- Mine blocks
- Support for different chains (Ethereum, Optimism)
- Utilities for contract deployment and interaction

### CosmWasm Testing

`CosmWasmContainer` provides a testing environment for CosmWasm contracts. It supports:

- Create isolated wasmd containers for CosmWasm testing
- Deploy and interact with CosmWasm contracts
- Initialize different types of contracts
- Execute contract methods and query contract state

### SatLayer Core Contracts Bootstrapping

The package includes utilities to bootstrap SatLayer core contracts for both EVM and CosmWasm environments:

- `EVMContracts` for EVM environments
  - Initialize `SLAY*` core contracts
  - Initialize ERC20 tokens and `SLAYVaults`
- `CosmWasmContracts` for CosmWasm environments
  - Initialize SatLayer core contracts (e.g. `bvs-registry`, `bvs-router`, etc.)
  - Initialize CW20 tokens
  - Initialize `bvs-vaults`

## Usage Examples

### Ethereum Testing

Look at [`anvil-container.test.ts`](./src/anvil-container.test.ts) for detailed examples of how to use the `AnvilContainer` class.

### CosmWasm Testing

Look at [`cosmwasm-container.test.ts`](./src/cosmwasm-container.test.ts) for detailed examples of how to use the `CosmWasmContainer` class.

### Bootstrapping SatLayer Core Contracts

Look at [`evm-contracts.test.ts`](./src/evm-contracts.test.ts) and [`cosmwasm-contracts.test.ts`](./src/cosmwasm-contracts.test.ts) for detailed examples of how to use the `EVMContracts` and `CosmWasmContracts` classes to bootstrap SatLayer core contracts in EVM and CosmWasm environments respectively.

## API Reference

### Ethereum (EVM)

- `AnvilContainer`: Creates and manages Anvil containers
- `StartedAnvilContainer`: Provides methods for interacting with a running container
- `EVMContracts`: Manages contract deployment and interaction

### CosmWasm

- `CosmWasmContainer`: Creates and manages wasmd containers
- `StartedCosmWasmContainer`: Provides methods for interacting with a running container
- `CosmWasmContracts`: Manages contract deployment and interaction

## EVMContracts

### initERC20

Initialize a mocked ERC20 contracts that supports:

- Customizable decimals
- Minting and burning tokens by anyone

```solidity
contract MockERC20 is ERC20 {
    uint8 private _decimals;

    /// @dev adds the decimals to be customizable
    constructor(string memory name_, string memory symbol_, uint8 decimals_) ERC20(name_, symbol_) {
        _decimals = decimals_;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external {
        _burn(from, amount);
    }
}
```
