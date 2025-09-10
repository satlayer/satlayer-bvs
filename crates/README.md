---
title: Contract Reference
asIndexPage: true
---

# CosmWasm Core Contracts

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
- **BVS Vault Bank Tokenized**:
  Specialized vault to support Bank module with tokenization of liquid shares.
- **BVS Vault Factory**:
  Enables operators to deploy new vault contracts with proper configuration.
- **BVS Vault Base**:
  Provides foundational components for all vault contracts,
  including share accounting (implementing the ERC-4626 standard), router integration, and security mechanisms.

## Contract Addresses

Individual vaults contract addresses are not listed here, as they are deployed by the operators themselves.

### Mainnet `bbn-1`

| Contract Name    | Address                                                          |
| :--------------- | :--------------------------------------------------------------- |
| BVS Pauser       | `bbn1mygmlglvg9w45n3s6m6d4txneantmupy0sa0vy63angpvj0qp7usr8wxgg` |
| BVS Registry     | `bbn1qtvnjezrv3fnqvuq869595zq6e2jk0zfhupg52aua0d6ht2a4jjsprqeae` |
| BVS Vault Router | `bbn1m2f0ctm657e22p843lgm9pnwlqtnuf3jgln7uyqrw6sy7nd5pc5qaasfud` |

### Testnet `bbn-test-5`

| Contract Name     | Address                                                          |
| :---------------- | :--------------------------------------------------------------- |
| BVS Pauser        | `bbn1e743905t9twc3cggaw7kxhcutls2lnlccg94gyv3c837x65ffglqmx3yg3` |
| BVS Registry      | `bbn1a9tleevqygn862ll2la49g637fjttfzzlttdrmmua35fadpuvnksuyud7a` |
| BVS Vault Router  | `bbn1tztx8vkgw24rm5f6ny52qyt6kpg7gyfd5nggvfgjjfj8n7ggkx7qfhvdum` |
| BVS Vault Factory | `bbn1cwm9fr8myw45zj5df5l4lrxyn03hleampj0a7nefztm5vt4w0r9qg2653p` |
