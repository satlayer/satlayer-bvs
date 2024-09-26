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

### Deployed Contract Addresses on Osmosis testnet

- strategy_base: osmo1arx9pfke8pc3x72xnf5rcthyq5zp2lu757x8j736a0jp5afe7ayq6v64sl
- strategy_manager: osmo12cqmhqdecwcl8eheawuhwnz9hcuagdfttm8rgjyh6ucfvdvt5gpqd2lz7u
- avs-directory: osmo1s06n72vlzzksu2fvlk0e2lm737nyvjrx2vtlv80gzmgzg7q5aayqfs9zme
- delegation_manager: osmo1f8c99usxzd8a0e4p02jcfldc4w4v97q96jxy9y9k9an6ka84ep3s6nnvnj
- state_bank: osmo1axj63dymth3sln5vptqy5dde28a8ptjvuukdpwrz44wp0lexgpmqpucn7k
- avs_driver: osmo1vxexrvx2prd33ny9ydh5ksnyt2nylcw0q87as5ph824jgxc3kcyqagrf9t
- rewards-coordinator: osmo1w3qsxj4smkfupts8vwtgn00f72qkpwns5jw0hexrnhv4vzw6gnmshus9l7
- cw20: osmo1z63lzuke2v9azva2kktccuecgcrm4ue4hhqrw0kfkf3crtxpqpdsyl8uj5

### Deployed Contract Addresses on Babylon testnet

- cw20: bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ, tx_hash: 9A2AC4E9DF6D9FFDDF0CD6B7F9D0B5AB1C5B274F93C38BE0CCB6F986CF6A610E

- slash-manager: bbn1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftq6wts0h, tx_hash: 1C822AE8EB476327D6B9A422BFF175CFB1FA725E68865B0AA412B7631FF138EB

- strategy-manager: bbn1436kxs0w2es6xlqpp9rd35e3d0cjnw4sv8j3a7483sgks29jqwgsdem28a, tx_hash: CC09135D3C56F2DFB1A8CF2F0C1060639853BC98EB0E3F0778174B9A657074FF

- strategy-base: bbn1mf6ptkssddfmxvhdx0ech0k03ktp6kf9yk59renau2gvht3nq2gqhwavk5, tx_hash: 7FCC6618CFECB973CFE041CD3660896E7A1177277F54DAB0815D6F689A692896

- delegation-manager: bbn1wn625s4jcmvk0szpl85rj5azkfc6suyvf75q6vrddscjdphtve8s4qaajp, tx_hash: 47EAD3092234AE563B72586EE189BF13143669456FC463D6A622F77CE7A912F4

- avs-directory: bbn1tqwwyth34550lg2437m05mjnjp8w7h5ka7m70jtzpxn4uh2ktsmq2s7xtm, tx_hash: BD38315A7C1BB050A3221B082721B8616EFF05F92590E07442BDEBD1F8F15C1E

- rewards-coordinator: bbn1gurgpv8savnfw66lckwzn4zk7fp394lpe667dhu7aw48u40lj6jspwu0t9, tx_hash: B5F2C0A487DA57E0CA4D3FBAD666F0B6C5C1DCFC5B471E1B6C6E2F6F4A639C5A

- strategy-factory: bbn1999u8suptza3rtxwk7lspve02m406xe7l622erg3np3aq05gawxsu7x00m, tx_hash: 65BE1A342E1465B2FF61E8A29C615C9B5CC7CE652C29D91E56118FB4DED4F44D

- strategy-base-TVLLimits: bbn1fventeva948ue0fzhp6xselr522rnqwger9wg7r0g9f4jemsqh6sv9ms6p, tx_hash: 1144EA6D894BC61F4AD34EBAAB3F8364AD5FAE4B5EF5AE61DC9679CEF2551783

- avs-driver: bbn18yn206ypuxay79gjqv6msvd9t2y49w4fz8q7fyenx5aggj0ua37qsgnhl9, tx_hash:
C0946703440EF1F665B0AB47B466555EA4CA08B8E86258DE5F34FECBF10B840D

- state-bank: bbn1l6z44fupg8l5mapmuxnrnmvxyclw5wnh5f5hfrtkaq5r0d9s24msp7np7z, tx_hash:
6DA57B4AFC4BBF8CBEA583CF8C61EEC2B68D69B7DAD45CA83E0FEE9E63CD3578