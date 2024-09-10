# Satlayer

This repository contains five smart contracts developed for the Babylon blockchain using Rust and CosmWasm. These contracts are designed to handle various functionalities within the Babylon ecosystem.

## Contracts

1. **AVSManager**

   - **Description**: The AVSManager contract manages the administration and validation of services within the Babylon chain.
   - **Purpose**: To provide robust administrative control and validation mechanisms.

2. **DelegationManager**

   - **Description**: The DelegationManager contract handles the delegation of tasks and roles.
   - **Purpose**: To streamline task delegation and role assignment processes.

3. **StrategyManager**

   - **Description**: The StrategyManager contract defines and manages operational strategies.
   - **Purpose**: To implement and oversee various strategic operations.

4. **SlashManager**

   - **Description**: The SlashManager contract is responsible for slashing misbehaving validators.
   - **Purpose**: To maintain the integrity and security of the network by penalizing faulty validators.

5. **RewardsCoordinator**

   - **Description**: The RewardsCoordinator contract manages the distribution of rewards to participants.
   - **Purpose**: To ensure fair and efficient distribution of rewards to contributors.

6. **StateBank**
   - **Description**: The StateBank contract just can be called by "set" method and emit UpdateState event.
   - **Purpose**: To UpdateState event so that it can be handled by offchain service.

## Development

These contracts are developed using Rust and CosmWasm, a powerful framework for developing smart contracts on the Cosmos SDK.

### Prerequisites

- Rust: Ensure you have Rust installed. You can install it from [rustup.rs](https://rustup.rs).
- CosmWasm: Follow the [CosmWasm documentation](https://docs.cosmwasm.com) to set up the environment.

### Building and Testing

Each contract is located in its own directory under the `contracts` directory. You can build and test the contracts using Cargo.

To build a contract:

```sh
cd contracts/AVSManager
cargo wasm
```

To build a contract:

```sh
cargo test
```

Optimizing Wasm
Use the provided script to optimize the Wasm output for deployment:

```sh
cargo run --package optimizer --release -- --input ./target/wasm32-unknown-unknown/release/avsmanager_contract.wasm --output ./target/wasm32-unknown-unknown/release/avsmanager_contract_optimized.wasm
```

### Deployed Contract Addresses

- strategy_base: osmo1arx9pfke8pc3x72xnf5rcthyq5zp2lu757x8j736a0jp5afe7ayq6v64sl
- strategy_manager: osmo12cqmhqdecwcl8eheawuhwnz9hcuagdfttm8rgjyh6ucfvdvt5gpqd2lz7u
- avs-directory: osmo1s06n72vlzzksu2fvlk0e2lm737nyvjrx2vtlv80gzmgzg7q5aayqfs9zme
- delegation_manager: osmo1f8c99usxzd8a0e4p02jcfldc4w4v97q96jxy9y9k9an6ka84ep3s6nnvnj
- state_bank: osmo1ha0wv4l764qwh4qs39duj9sutraqntqhrj9vcgpcpgca3z6sjmtsyrpqh6
- rewards-coordinator: osmo1w3qsxj4smkfupts8vwtgn00f72qkpwns5jw0hexrnhv4vzw6gnmshus9l7
- cw20: osmo1z63lzuke2v9azva2kktccuecgcrm4ue4hhqrw0kfkf3crtxpqpdsyl8uj5
