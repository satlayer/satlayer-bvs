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

- strategy_base: osmo18yvq860y5wl5gaaggfjewpuma3wfamfv7hje90l8h2eq4x6pcdascx87ss
- strategy_manager: osmo138sme44hvff0psva7harc806j7tkcfxue24hyz5du9x92d5urn9snpffah
- avs-directory: osmo1txnudh57gm69p0y9h838mezk2y29avgrggrym3xykp83fcrahsuqwlpewh
- delegation_manager: osmo15df3cz842rm5rx9xqt9r08ppq5jwkvzlh48tfctjgcpe5vedszdq32x0qr
- state_bank: osmo1ha0wv4l764qwh4qs39duj9sutraqntqhrj9vcgpcpgca3z6sjmtsyrpqh6
