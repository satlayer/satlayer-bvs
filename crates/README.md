---
title: Babylon Contracts
---

# Babylon Contracts (CosmWasm)

SatLayer BVS is a comprehensive ecosystem on the Babylon network
that includes a suite of interconnected contracts for managing staking,
delegation, slashing, rewards distribution, and automated validator selection.

## Contracts

### BVS Directory

The BVS Directory contract is responsible
for managing the registration and deregistration of operators with Bitcoin Validated Service
(BVS) within the BVS ecosystem.
It enables BVS to register their services and allows operators to securely register or deregister with a BVS.
The contract ensures the integrity and security of the registration process through digital signature verification,
the use of unique salts to prevent replay attacks, and expiration checks on signatures.
Additionally, it offers functionality to update BVS metadata URIs,
transfer ownership of the contract, and pause or unpause contract operations.

### BVS Delegation Manager

The DelegationManager contract is a core component of the BVS ecosystem
that facilitates the delegation of staker shares to operators.
It allows stakers to delegate their shares from various strategies to trusted operators,
who can manage staking operations on their behalf.
The contract handles the registration of operators, including their details and metadata,
and enforces delegation through signature verification to ensure security.
It manages the delegation lifecycle, including delegating shares, undelegating,
and queuing withdrawals with configurable delay blocks for security purposes.
The contract integrates with other system components like the StrategyManager and SlashManager
to coordinate share management,
enforce slashing penalties, and maintain the integrity of the staking process.
By providing mechanisms for controlled delegation and secure withdrawal processes,
the DelegationManager enhances the flexibility and efficiency of staking operations within the ecosystem.

### BVS Strategy Manager

The StrategyManager contract serves as a central hub in the BVS ecosystem,
facilitating stakers' interactions with various investment strategies.
It manages the processes of depositing tokens into whitelisted strategies,
tracking staker shares, and enabling withdrawals of shares as tokens.
The contract ensures secure and efficient management of staker assets
by maintaining accurate share accounting and enforcing strict access controls.
It supports off-chain approvals through signature verification, integrates with delegation and slashing mechanisms,
and allows for the dynamic addition or removal of strategies from the whitelist by authorized entities.
The StrategyManager enhances the overall functionality and security of staking and investment operations within the network
by providing a standardized and secure interface for stakers to engage with approved strategies.

### BVS Strategy Base

The StrategyBase contract serves as a foundational implementation for investment strategies within the BVS ecosystem.
It provides core functionalities for depositing and withdrawing underlying tokens,
calculating shares, and managing the total shares of the strategy.
This contract handles the conversion between underlying tokens and strategy shares
using a virtual balance mechanism to mitigate rounding errors and improve precision.
It also includes mechanisms to pause deposits and withdrawals,
transfer ownership, and emit events related to exchange rates.

### BVS Strategy TVL Limits

The StrategyTVLLimits contract is an extension of the base strategy implementation within the BVS ecosystem.
It introduces Total Value Locked (TVL) limits to the strategy,
allowing the contract owner to set maximum limits on individual deposits and the total deposits in the strategy.
This contract maintains core functionalities such as depositing and withdrawing underlying tokens,
calculating shares, and managing the total shares of the strategy.
It includes mechanisms to pause deposits and withdrawals, transfer ownership, and emit events related to exchange rates.
The addition of TVL limits provides an extra layer of control over the strategy's growth and risk exposure.

### BVS Strategy Factory

The StrategyFactory contract serves as a centralized factory
for deploying and managing investment strategy contracts within the BVS ecosystem.
It allows the contract owner to create new strategy contracts for specific tokens,
manage existing strategies, and enforce policies such as blacklisting tokens and controlling strategy access.
The contract interacts with a StrategyManager contract to maintain a whitelist of approved strategies
and to set permissions related to third-party transfers.
It also includes functionalities for pausing the deployment of new strategies, transferring ownership,
and updating configuration parameters like the strategy code ID and the strategy manager address.

### BVS Slash Manager

The SlashManager contract is a critical component in the BVS ecosystem
responsible for enforcing slashing penalties on operators and their associated stakers for protocol violations or misconduct.
Authorized entities known as slashers can submit and execute slash requests,
specifying the operator to be penalized, the amount of shares to reduce, and the reason for the slashing.
The contract validates these requests,
ensuring they meet required criteria such as minimum signature thresholds and validator approvals.
Upon validation,
it interacts with the DelegationManager and StrategyManager contracts
to proportionally reduce the shares of affected stakers,
effectively implementing the penalty.
This mechanism helps
maintain network integrity and security
by deterring malicious behavior and ensuring compliance with protocol rules.

### BVS Rewards Coordinator

The RewardsCoordinator contract is responsible for managing the distribution
and claiming of rewards within the BVS ecosystem.
It allows authorized entities to submit reward distributions in the form of Merkle roots,
which represent the rewards allocated to earners over specific periods.
Participants can claim their rewards
by providing Merkle proofs that verify their entitlement within these distributions.
The contract ensures that only legitimate and authorized claims are processed,
thereby maintaining the integrity and fairness of the reward system.
By coordinating the distribution and secure claiming of rewards,
the RewardsCoordinator plays a crucial role in incentivizing participation
and upholding the fairness of the network's reward mechanisms.

### BVS State Bank

The StateBank contract functions as a simple on-chain key-value store within the BVS ecosystem.
It allows registered Bitcoin Validated Service (BVS)
contracts to securely store and update integer values associated with specific string keys.
Only BVS contracts that have been registered with the StateBank can modify the stored values,
ensuring that only authorized entities have write access.
This mechanism enhances security by preventing unauthorized modifications while promoting transparency,
as any user can query the stored values by providing the corresponding key.
The StateBank thus provides a reliable and straightforward way for BVS contracts
to persist and share state information on the blockchain,
supporting the overall functionality and integrity of the ecosystem.

### BVS Driver

The BVS Driver contract serves as an interface for Bitcoin Validated Service (BVS)
contracts within the BVS ecosystem to securely initiate off-chain tasks.
It maintains a registry of authorized BVS contracts,
allowing only registered contracts to trigger off-chain executions via the execute_bvs_offchain function,
which includes a task_id identifying the specific task.
The contract emits events containing the sender's address and the task ID,
enabling off-chain services to monitor and execute the corresponding tasks.
By enforcing strict access control and providing a standardized mechanism for initiating off-chain operations,
the BVS Driver enhances the security and reliability of interactions between on-chain contracts and off-chain services in the network.
