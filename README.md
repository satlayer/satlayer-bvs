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
- strategy_manager: osmo1p7lcc7drqdc4qlv3v5j9wd3hjey8wylyn9rtqxygl7tdxhcqvn2qhfwnt2
- avs-directory: osmo1s06n72vlzzksu2fvlk0e2lm737nyvjrx2vtlv80gzmgzg7q5aayqfs9zme
- delegation_manager: osmo1uplxqr00x57dd2dt9j3vnfmhxz5j9qwn2dyv6587exgsf0zpwjkqpzuzm0
- state_bank: osmo1axj63dymth3sln5vptqy5dde28a8ptjvuukdpwrz44wp0lexgpmqpucn7k
- avs_driver: osmo1vxexrvx2prd33ny9ydh5ksnyt2nylcw0q87as5ph824jgxc3kcyqagrf9t
- rewards-coordinator: osmo1w3qsxj4smkfupts8vwtgn00f72qkpwns5jw0hexrnhv4vzw6gnmshus9l7
- cw20: osmo1z63lzuke2v9azva2kktccuecgcrm4ue4hhqrw0kfkf3crtxpqpdsyl8uj5
- strategy-base-TVLLimits: osmo1fzu4779d8yfgs5dw8k8upx5fqj3e38cewqhthpym47rrmpwjq40sc7ptdm
- strategy-factory: osmo1f2aup80qnq3knlkjgcm98pcakvy2x3u6jn4att5makpflpptspdq8fk37g
- slash-manager: osmo1xu6xnd47qay5dqaujlvqg6m43y2gjza3w0urfj0e2zau74mj9dlq90pxdl

### Deployed Contract Addresses on Babylon testnet

- cw20: bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ, tx_hash: 9A2AC4E9DF6D9FFDDF0CD6B7F9D0B5AB1C5B274F93C38BE0CCB6F986CF6A610E

- slash-manager: bbn1d4sad30uj59lxg56ylwn7457v8z4k5m3323r9u360q85w8ga3kfsghm2sx, tx_hash: 75950867AAC6AD413AA9593C57B96F8A00AED62DF7790A4CC673DA9756C4466B

- strategy-manager: bbn1hyja4uyjktpeh0fxzuw2fmjudr85rk2qu98fa6nuh6d4qru9l0ssdhxckk, tx_hash: 0F0F16B50C25F73A80BBDB10711DF1C45AAEDB432F7E2F4A1F4153D8345703F5

- strategy-base1: bbn1fxkx7wd4q7z5zgm8qh0vaxptx4gp0ppgjm0ke56jr55azpzecpcsru2d8p, tx_hash: 470167FEAEE173BEB8A452DA44F926C088399297825725E71F3E9330D8D921BB

- strategy-base2: bbn105w0chel9d69cdzj62m4h2vqtj6vfuh7yzty2en825t2nyxw74es4xcf7s, tx_hash: E444322972DF19310F93B6A06BC9A88D73E7B3AD2AF2B034AD3B764A56613F99

- delegation-manager: bbn18glh4zetf3nkdu724dxqvlw2gw6fdwnhrycazt32dgysq5gvyj4sd5n53x, tx_hash: 8A82789CA76116218193A13F107E0420470FB2888B9F923E357C18F88A8FB9E9

- avs-directory: bbn13v6dgzhf9nu4fzdkrc6tpvxxd8eqg546ynjep8cqvl4n27xlvf7suqvspx, tx_hash: 0C44390F32198AB04165A73B1037AAC2AF8984D02C39D56F9765407EA4031D18

- rewards-coordinator: bbn1w95u2w477a852mpex72t4u0qs0vyjkme4gq4m2ltgf84km47wf0sxzlx3w, tx_hash: 8D89CAB034D7956ABC269E469AFBC2764994C373DC4847B4E50C9691427525FE

- strategy-factory: bbn1p4lqunauqgstt6ydszx59y3pg2tkaxlnujl9m5ldz7nqcrn6tjzqsqv454, tx_hash: 5DCE08D2631116AAF7DB6401BF92F88861C9CEEDEF590073B27C8D5253465D80

- strategy-base-TVLLimits: bbn1m85zj4fm3u6zmcj950rj6mjwuv54r5et9al8yhcee5j3ua8keqyst7zld4, tx_hash: A51F6EDFF7E7110C0C011677B3EAF24861A61F7B809710B507CEB60B2B99FA13

- avs-driver: bbn1heufwzln70cc9c6tpj45nsn02pm7xjffgmy5h7gvs3c57ndqr8dskx53lf, tx_hash:
F1F850A839262D896919538F25838635ECC901A038B2C3D9AC9E65A24BBDCEED

- state-bank: bbn18nnde5tpn76xj3wm53n0tmuf3q06nruj3p6kdemcllzxqwzkpqzq5mssn4, tx_hash:
EFA69375C9961B51D8814E0BFE9D14E9ED45E0AD9E04ADAE7D33677BFE846CBE