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

- strategy_base: osmo1p4ee54wcu54vcxht5spk5dpklr39qjpxxk38rm9p36c48rlgyawstwl3q8
- strategy_manager: osmo18hyk6uy03amrsxfcrtahjl07c536plucc27ds259lp3gz3w00h5sntmdsr
- avs-directory: osmo1rrcshedc44wanwyzfeklku4vdqu5r5d87j0tdektjyeudnw2j3rsffg6ux
- delegation_manager: osmo1l08utjv2vq0hg3hjvcn97eypxl0za22js8auczqt7xphk7ggcnmqfnsnct
