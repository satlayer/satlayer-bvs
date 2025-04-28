---
title: Core Contracts
asIndexPage: true
---

# Core Contracts

The SatLayer ecosystem consists of several specialized contracts
that work together to provide a complete staking and validation solution.
These contracts are designed to be modular and extensible to actively validate a suite of services.
The contracts are built primarily in Rust, using the CosmWasm framework.

**Core**

- **BVS Registry**:
  A central record-keeping contract for all Operators and Services within the ecosystem, tracking mutual relationships and validation status.
- **BVS Vault Router**:
  The central entry/exit point for all vaults, managing whitelist status and operator-vault relationships.
- **BVS Pauser**:
  Provides emergency pause functionality for the ecosystem's contracts.
- **BVS Rewards**:
  A contract that manages the distribution of rewards to operators and restakers.

**Vaults**

- **BVS Vault Bank**:
  Specialized vault for native bank tokens (e.g., `ubbn`).
- **BVS Vault CW20**:
  Specialized vault to support CW20 tokens.
- **BVS Vault CW20 Tokenized**:
  Specialized vault to support CW20 tokens with tokenization of liquid shares.
- **BVS Vault Factory**:
  Enables operators to deploy new vault contracts with proper configuration.
- **BVS Vault Base**:
  Provides foundational components for all vault contracts,
  including share accounting (implementing the ERC-4626 standard), router integration, and security mechanisms.

### Contract Relationships

The SatLayer BVS contracts form an interconnected ecosystem:

1. **Registry-Router Relationship**:
   The Vault Router queries the Registry to determine if operators are validating services.
2. **Router-Vault Relationship**:
   Vaults check with the Router to determine if they're whitelisted and if operators are validating.
3. **Factory-Vault Relationship**:
   The Factory deploys new vaults with proper configuration.
4. **Pauser Integration**:
   Critical contracts integrate with the Pauser for emergency functionality.
