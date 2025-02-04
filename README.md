# SatLayer BVS

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

- strategy_base1: osmo1j89xj4hlllgpy8jw4l5wlza6wcxdfhkvnzv22wl4cn69pjd2fhsqucfp7l
- strategy_base2: osmo1c5yly86d40cg9ppxeajlqgsx488tauf6ssgvxvvuaulcl0s76jws9krdeq
- strategy_manager: osmo1ayukz2qfgp77yv3uscpd4xf97zpuax4ejnv2adw652qw9x50avzsyfa2q4
- bvs-directory: osmo1c8hrsyncq8w95gwvxx8yh4lkl8syrnf0qwttx2f6dsmlxt88prsq79fpuz
- delegation_manager: osmo1zr7kmhn7s62jsk4nrlsasuuvkec9sanksqqkps2gf5tlw9stx6zsnrs200
- state_bank: osmo14me62ahp32xrkrqnllmsfthfzqxgf0xqshxtk5ghdfwjltdjh2pqdhn8j9
- bvs_driver: osmo14rrkya0p6h0xf8v3f33grp6dv7lqs2r5xg09zpzjgnggjgfc08fs9kz9ru
- rewards-coordinator: osmo1rnv0dzld0agh9akt92dttewkuyfj6753ew5k49rscfdc04as5krqn8pufs
- cw20: osmo1sahxe3sylexuga6mz6qssmcqempk7rknge0amyj6dynu2gq0gmeqnvr60s
- strategy-base-TVLLimits: osmo12wq3w24nqetleqjgpyeth5uj7280mmwcdukdl62n06cz6l8jtmzsarr7gh
- strategy-factory: osmo144wl7rs58hq9tz2z2y0e62j34ghndfwvt2pesjg0ps0s0d5x28xs5vcled
- slash-manager: osmo1l6masaw2z9d7tvhtjenahmgeqsp8v24apncp9pxq4zdflqh3za6sn786ge

### Deployed Contract Addresses on Babylon testnet

- cw20: bbn1mx295r0mph0xvetqqcapsj4xxreg9mek7nlzhcacu4y0r83hhxfqu9mn0v
- cross-chain-cw20: bbn1rtn4eh58eta2ehnm2vgnj2z9u9f8zt8swzuetxud26ff5u70l5cslcs2wr
- slash-manager: bbn1kyjqfdnnmlqe0xt78rg4wtpfyhznzc60887rfuf2kr079hga3htqgzjhxw
- strategy-manager: bbn1txnzllunz2syce9kdg6zev4g2t90afz53xs944vcan9pl7flp2lqfkuq62
- strategy-base1: bbn1dkqqk0pskzw5ptxatxwfatqk2z3398vkcpvefhg050tqhxzxlnxqvaf9h0
- strategy-base2: bbn14rruau4y52cqyag6d9pxa3rrwhhh9xu7egndpafu55ztd8dprj8s860s8w
- delegation-manager: bbn1e6wth7499p3y4y7h45nx4dkjlm62gn8shgjtrezymdepz58ne3tq58hkmv
- bvs-directory: bbn1hr2m4e6jwlplhwq0r8khyqnvvsquvsd8u44hmn0k4v99rcrmgd0slhmxn7
- rewards-coordinator: bbn1xwpk5mrrrm7zsl606mhdj5lmtmegcu9c72ve7hyd7kf7n3v2jnrq2wgyxf
- strategy-factory: bbn1x7v4jf9ezmy9zy7yzjqv4njy0ef3q8np0dey6agj67wznsa90zdslzvyxs
- strategy-base-TVLLimits: bbn108l2c6l5aw0pv68mhq764kq9344h4prefft4uufelmweasfstfzsxv0w5p
- bvs-driver: bbn18x5lx5dda7896u074329fjk4sflpr65s036gva65m4phavsvs3rqk5e59c
- state-bank: bbn1h9zjs2zr2xvnpngm9ck8ja7lz2qdt5mcw55ud7wkteycvn7aa4pqpghx2q
- mirrored-token: bbn1hcg2jce8s3zp5t7c2d728hzudeef7jgp78ncd0sjz8xwzp75qm3qfr26q3
