# SatLayer

 SatLayer is a comprehensive blockchain ecosystem on the Babylon network that includes a suite of interconnected contracts for managing staking, delegation, slashing, rewards distribution, and automated validator selection (BVS). The StrategyManager facilitates staker interactions with investment strategies; the DelegationManager manages the delegation of staker shares to operators; the SlashManager enforces slashing penalties for protocol violations; the RewardsCoordinator oversees the distribution and claiming of rewards; the StateBank provides a key-value store for BVS contracts; and the BVS Driver enables secure off-chain task execution for BVS contracts. Together, these contracts create a secure and efficient system for staking, delegating, rewarding, and selecting validators within the Babylon blockchain ecosystem.

## Contracts

1. **BVSDirectory**

   - **Description**: The BVSDirectory contract is responsible for managing the registration and deregistration of operators with Assured Verification Services (BVS) within the Babylon blockchain ecosystem. It enables BVS providers to register their services and allows operators to securely register or deregister with an BVS. The contract ensures the integrity and security of the registration process through digital signature verification, the use of unique salts to prevent replay attacks, and expiration checks on signatures. Additionally, it offers functionality to update BVS metadata URIs, transfer ownership of the contract, and pause or unpause contract operations.

2. **DelegationManager**

   - **Description**: The DelegationManager contract is a core component of the Babylon blockchain ecosystem that facilitates the delegation of staker shares to operators. It allows stakers to delegate their shares from various strategies to trusted operators, who can manage staking operations on their behalf. The contract handles the registration of operators, including their details and metadata, and enforces delegation through signature verification to ensure security. It manages the delegation lifecycle, including delegating shares, undelegating, and queuing withdrawals with configurable delay blocks for security purposes. The contract integrates with other system components like the StrategyManager and SlashManager to coordinate share management, enforce slashing penalties, and maintain the integrity of the staking process. By providing mechanisms for controlled delegation and secure withdrawal processes, the DelegationManager enhances the flexibility and efficiency of staking operations within the Babylon network.

3. **StrategyManager**

   - **Description**: The StrategyManager contract serves as a central hub in the Babylon blockchain ecosystem, facilitating stakers' interactions with various investment strategies. It manages the processes of depositing tokens into whitelisted strategies, tracking staker shares, and enabling withdrawals of shares as tokens. The contract ensures secure and efficient management of staker assets by maintaining accurate share accounting and enforcing strict access controls. It supports off-chain approvals through signature verification, integrates with delegation and slashing mechanisms, and allows for the dynamic addition or removal of strategies from the whitelist by authorized entities. The StrategyManager enhances the overall functionality and security of staking and investment operations within the network by providing a standardized and secure interface for stakers to engage with approved strategies.

4. **StrategyBase**

   - **Description**: The StrategyBase contract serves as a foundational implementation for investment strategies within the Babylon blockchain ecosystem. It provides core functionalities for depositing and withdrawing underlying tokens, calculating shares, and managing the total shares of the strategy. This contract handles the conversion between underlying tokens and strategy shares using a virtual balance mechanism to mitigate rounding errors and improve precision. It also includes mechanisms to pause deposits and withdrawals, transfer ownership, and emit events related to exchange rates.

5. **StrategyTVLLimits**

   - **Description**: The StrategyTVLLimits contract is an extension of the base strategy implementation within the Babylon blockchain ecosystem. It introduces Total Value Locked (TVL) limits to the strategy, allowing the contract owner to set maximum limits on individual deposits and the total deposits in the strategy. This contract maintains core functionalities such as depositing and withdrawing underlying tokens, calculating shares, and managing the total shares of the strategy. It includes mechanisms to pause deposits and withdrawals, transfer ownership, and emit events related to exchange rates. The addition of TVL limits provides an extra layer of control over the strategy's growth and risk exposure.

6. **StrategyFactory**

   - **Description**: The StrategyFactory contract serves as a centralized factory for deploying and managing investment strategy contracts within the Babylon blockchain ecosystem. It allows the contract owner to create new strategy contracts for specific tokens, manage existing strategies, and enforce policies such as blacklisting tokens and controlling strategy access. The contract interacts with a StrategyManager contract to maintain a whitelist of approved strategies and to set permissions related to third-party transfers. It also includes functionalities for pausing the deployment of new strategies, transferring ownership, and updating configuration parameters like the strategy code ID and the strategy manager address.

7. **SlashManager**

   - **Description**: The SlashManager contract is a critical component in the Babylon blockchain ecosystem responsible for enforcing slashing penalties on operators and their associated stakers for protocol violations or misconduct. Authorized entities known as slashers can submit and execute slash requests, specifying the operator to be penalized, the amount of shares to reduce, and the reason for the slashing. The contract validates these requests, ensuring they meet required criteria such as minimum signature thresholds and validator approvals. Upon validation, it interacts with the DelegationManager and StrategyManager contracts to proportionally reduce the shares of affected stakers, effectively implementing the penalty. This mechanism helps maintain network integrity and security by deterring malicious behavior and ensuring compliance with protocol rules.

8. **RewardsCoordinator**

   - **Description**: The RewardsCoordinator contract is responsible for managing the distribution and claiming of rewards within the Babylon blockchain ecosystem. It allows authorized entities to submit reward distributions in the form of Merkle roots, which represent the rewards allocated to earners over specific periods. Participants can claim their rewards by providing Merkle proofs that verify their entitlement within these distributions. The contract ensures that only legitimate and authorized claims are processed, thereby maintaining the integrity and fairness of the reward system. By coordinating the distribution and secure claiming of rewards, the RewardsCoordinator plays a crucial role in incentivizing participation and upholding the fairness of the network's reward mechanisms.

9. **StateBank**
    
   - **Description**: The StateBank contract functions as a simple on-chain key-value store within the Babylon blockchain ecosystem. It allows registered Automated Validator Selection (BVS) contracts to securely store and update integer values associated with specific string keys. Only BVS contracts that have been registered with the StateBank can modify the stored values, ensuring that only authorized entities have write access. This mechanism enhances security by preventing unauthorized modifications while promoting transparency, as any user can query the stored values by providing the corresponding key. The StateBank thus provides a reliable and straightforward way for BVS contracts to persist and share state information on the blockchain, supporting the overall functionality and integrity of the ecosystem.

10. **BVSDriver**
    
    - **Description**: The BVS Driver contract serves as an interface for Automated Validator Selection (BVS) contracts within the Babylon blockchain ecosystem to securely initiate off-chain tasks. It maintains a registry of authorized BVS contracts, allowing only registered contracts to trigger off-chain executions via the execute_bvs_offchain function, which includes a task_id identifying the specific task. The contract emits events containing the sender's address and the task ID, enabling off-chain services to monitor and execute the corresponding tasks. By enforcing strict access control and providing a standardized mechanism for initiating off-chain operations, the BVS Driver enhances the security and reliability of interactions between on-chain contracts and off-chain services in the network.

## Development

These contracts are developed using Rust and CosmWasm, a powerful framework for developing smart contracts on the Cosmos SDK.

### Prerequisites

- Rust: Ensure you have Rust installed. You can install it from [rustup.rs](https://rustup.rs).
- CosmWasm: Follow the [CosmWasm documentation](https://docs.cosmwasm.com) to set up the environment.

### Building and Testing

Each contract is located in its own directory under the `contracts` directory. You can build and test the contracts using Cargo.

Unit test contracts:

```sh
cargo test
```

Build custom cosmwasm optimizer with docker

```sh
sudo docker build -t custom-cosmwasm-optimizer -f Dockerfile .
```

Generate and optimizing Wasm:

```sh
docker run --rm -v "$(pwd)":/code custom-cosmwasm-optimizer:latest
```

### Deployed Contract Addresses on Osmosis testnet

- strategy_base1: osmo1xy70s05t52fnyzvd8z3hn99nfcqrrcds84gm9rzhfrryltt3kz6sxlnkxy
- strategy_base2: osmo1w9d24cv5ve4xclx7usrvdgyu35y6ytk2v5rjgc6uesyyunfc4snsvlc8nc
- strategy_manager: osmo1p7lcc7drqdc4qlv3v5j9wd3hjey8wylyn9rtqxygl7tdxhcqvn2qhfwnt2
- bvs-directory: osmo1s06n72vlzzksu2fvlk0e2lm737nyvjrx2vtlv80gzmgzg7q5aayqfs9zme
- delegation_manager: osmo1uplxqr00x57dd2dt9j3vnfmhxz5j9qwn2dyv6587exgsf0zpwjkqpzuzm0
- state_bank: osmo1axj63dymth3sln5vptqy5dde28a8ptjvuukdpwrz44wp0lexgpmqpucn7k
- bvs_driver: osmo1vxexrvx2prd33ny9ydh5ksnyt2nylcw0q87as5ph824jgxc3kcyqagrf9t
- rewards-coordinator: osmo1w3qsxj4smkfupts8vwtgn00f72qkpwns5jw0hexrnhv4vzw6gnmshus9l7
- cw20: osmo1z63lzuke2v9azva2kktccuecgcrm4ue4hhqrw0kfkf3crtxpqpdsyl8uj5
- strategy-base-TVLLimits: osmo1fzu4779d8yfgs5dw8k8upx5fqj3e38cewqhthpym47rrmpwjq40sc7ptdm
- strategy-factory: osmo1j7dpzhxq7kt6s59zqcdkv7e7fuz5rv5jtp3x5w9yejllan07nhtsc6d4tx
- slash-manager: osmo1xu6xnd47qay5dqaujlvqg6m43y2gjza3w0urfj0e2zau74mj9dlq90pxdl

### Deployed Contract Addresses on Babylon testnet

- cw20: bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ, tx_hash: 9A2AC4E9DF6D9FFDDF0CD6B7F9D0B5AB1C5B274F93C38BE0CCB6F986CF6A610E

- slash-manager: bbn1d4sad30uj59lxg56ylwn7457v8z4k5m3323r9u360q85w8ga3kfsghm2sx, tx_hash: 75950867AAC6AD413AA9593C57B96F8A00AED62DF7790A4CC673DA9756C4466B

- strategy-manager: bbn1hyja4uyjktpeh0fxzuw2fmjudr85rk2qu98fa6nuh6d4qru9l0ssdhxckk, tx_hash: 0F0F16B50C25F73A80BBDB10711DF1C45AAEDB432F7E2F4A1F4153D8345703F5

- strategy-base1: bbn1fxkx7wd4q7z5zgm8qh0vaxptx4gp0ppgjm0ke56jr55azpzecpcsru2d8p, tx_hash: 470167FEAEE173BEB8A452DA44F926C088399297825725E71F3E9330D8D921BB

- strategy-base2: bbn105w0chel9d69cdzj62m4h2vqtj6vfuh7yzty2en825t2nyxw74es4xcf7s, tx_hash: E444322972DF19310F93B6A06BC9A88D73E7B3AD2AF2B034AD3B764A56613F99

- delegation-manager: bbn18glh4zetf3nkdu724dxqvlw2gw6fdwnhrycazt32dgysq5gvyj4sd5n53x, tx_hash: 8A82789CA76116218193A13F107E0420470FB2888B9F923E357C18F88A8FB9E9

- bvs-directory: bbn13v6dgzhf9nu4fzdkrc6tpvxxd8eqg546ynjep8cqvl4n27xlvf7suqvspx, tx_hash: 0C44390F32198AB04165A73B1037AAC2AF8984D02C39D56F9765407EA4031D18

- rewards-coordinator: bbn1w95u2w477a852mpex72t4u0qs0vyjkme4gq4m2ltgf84km47wf0sxzlx3w, tx_hash: 8D89CAB034D7956ABC269E469AFBC2764994C373DC4847B4E50C9691427525FE

- strategy-factory: bbn1p4lqunauqgstt6ydszx59y3pg2tkaxlnujl9m5ldz7nqcrn6tjzqsqv454, tx_hash: 5DCE08D2631116AAF7DB6401BF92F88861C9CEEDEF590073B27C8D5253465D80

- strategy-base-TVLLimits: bbn1m85zj4fm3u6zmcj950rj6mjwuv54r5et9al8yhcee5j3ua8keqyst7zld4, tx_hash: A51F6EDFF7E7110C0C011677B3EAF24861A61F7B809710B507CEB60B2B99FA13

- bvs-driver: bbn1heufwzln70cc9c6tpj45nsn02pm7xjffgmy5h7gvs3c57ndqr8dskx53lf, tx_hash:
F1F850A839262D896919538F25838635ECC901A038B2C3D9AC9E65A24BBDCEED

- state-bank: bbn18nnde5tpn76xj3wm53n0tmuf3q06nruj3p6kdemcllzxqwzkpqzq5mssn4, tx_hash:
EFA69375C9961B51D8814E0BFE9D14E9ED45E0AD9E04ADAE7D33677BFE846CBE