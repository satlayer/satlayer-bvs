# BVS Rewards Coordinator

`bvs-rewards-coordinator` is responsible for holding the rewards from BVS and distributing them to the earners.

## General Flow

`Owner` sets the `RewardsUpdater` as init process.

`BVS` computes rewards off-chain, creates a reward merkle tree where the leaves are the rewards for each `Earner`.
The root of the merkle tree is passed to `RewardsUpdater`.

`BVS` also transfers the rewards (CW20 tokens) to `bvs-rewards-coordinator` contract through `create_bvs_rewards_submission` or proxy through setting `REWARDS_FOR_ALL_SUBMITTER`.

The `RewardsUpdater` submits the root of the rewards merkle tree to the `bvs-rewards-coordinator`.

The `Earner` sets the `Claimer` for their rewards, which could be themselves (`Claimer = Earner`) or not.

The `Claimer` calls `process_claim` with the reward root to claim, leaf and merkle proof.
This can only be done after the root is submitted + `activation_delay`.
Then `bvs-rewards-coordinator` transfers the rewards to the `Recipient` (passed in `process_claim` param).

```mermaid
flowchart TD
    subgraph 1["Flow w/ create_bvs_rewards_submission"]
    %% Nodes
    1.A[Owner]
    1.B[(bvs-rewards-coordinator)]
    1.C[RewardsUpdater]
    1.D[Earner]
    1.E[Claimer]
    1.F[BVS]
    1.G[(CW20)]
    1.H[Recipient]
    %% Padding to prevent overlap in text
    1.Z:::hidden ~~~ 1.A

    1.A ---> |"(1) set_rewards_updater"| 1.C
    1.F ---> |"(2.1) create_bvs_rewards_submission"| 1.B
    1.G ---> |"(2.2) CW20::TransferFrom"| 1.B
    1.C ---> |"(3) submit_root"| 1.B
    1.D ---> |"(4.1) set_claimer_for"| 1.E
    1.E ---> |"(4.2) process_claim"| 1.B
    1.G ---> |"(4.3) CW20::Transfer"|1.H
    end
```

```mermaid
flowchart TD
    subgraph 2["Flow w/ create_rewards_for_all_submission"]
    %% Nodes
    2.A[Owner]
    2.B[(bvs-rewards-coordinator)]
    2.C[RewardsUpdater]
    2.D[Earner]
    2.E[Claimer]
    2.G[REWARDS_FOR_ALL_SUBMITTER]
    2.H[(CW20)]
    2.I[Recipient]
    %% Padding to prevent overlap in text
    2.Z:::hidden ~~~ 2.A

    2.A ---> |"(1) set_rewards_updater"| 2.C
    2.A ---> |"(2) set_rewards_for_all_submitter"|2.G
    2.G ---> |"(3.1) create_rewards_for_all_submission"| 2.B
    2.H ---> |"(3.2) CW20::TransferFrom"| 2.B
    2.C ---> |"(4) submit_root"| 2.B
    2.D ---> |"(5.1) set_claimer_for"| 2.E
    2.E ---> |"(5.2) process_claim"| 2.B
    2.H ---> |"(5.3) CW20::Transfer"|2.I
    end
```
