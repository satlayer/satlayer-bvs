# BVS Rewards

The BVS Rewards contract manages the distribution and claiming of rewards in the SatLayer ecosystem.
It enables Services to distribute rewards to Operators and Stakers
(collectively called Earners) based on their contributions to the network.

## Contract Functions

### Distribution Functions

- `DistributeRewards`: Services distribute rewards by posting a Merkle root
  - Requires sufficient token allowance (CW20) or funds (Bank)
  - Updates the current Merkle root for the specified Service and token
  - Sends the specified number of tokens from the service to the contract

### Claiming Functions

- `ClaimRewards`: Earners claim rewards by providing a Merkle proof for the (service, token) pair
  - Verifies the proof against the stored Merkle root
  - Transfers the claimed rewards to the recipient
  - Updates claimed rewards tracking

### Query Functions

- `DistributionRoot`: Query the Merkle root for a Service and token
- `Balance`: Query the rewards balance available for a Service and token
- `ClaimedRewards`: Query the claimed rewards for a Service, token, and Earner

## Rewards Workflow

Refer to the [Rewards](/getting-started/rewards) section for a detailed overview of how rewards are distributed,
and claimed in the SatLayer ecosystem.

## Important Notes

- Non-standard CW20 tokens (e.g., with fee-on-transfer mechanisms) are not supported
- The contract maintains state for distribution roots, balances, and claimed rewards
- Rewards amount is cumulative
