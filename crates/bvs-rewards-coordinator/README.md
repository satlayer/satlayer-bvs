# BVS Rewards Coordinator

`bvs-rewards-coordinator` is responsible for holding the rewards from BVS and distributing them to the earners.

## General Flow

1. **Initial Setup**:

   - `Owner` configures `RewardsUpdater`

2. **Rewards Submission**:

   - `Service` calculates off-chain rewards for each `Strategy`
   - Sends tokens to contract via `create_rewards_submission`

3. **Rewards Activation**:

   - `RewardsUpdater` processes `RewardsSubmission` events periodically
   - Creates a cumulative rewards merkle tree with `(earner, token)` pairs
   - Submits root via `submit_root`

4. **Claiming Process**:
   - (optional) `Earner` designates a `Claimer`
   - `Claimer` executes `process_claim` with merkle proof after activation delay
   - Contract transfers rewards to specified `Recipient`

```mermaid
flowchart TD
    subgraph 1["Flow w/ create_rewards_submission"]
    %% Nodes
    1.A[Owner]
    1.B[(bvs-rewards-coordinator)]
    1.C[RewardsUpdater]
    1.D[Earner]
    1.E[Claimer]
    1.F[Service]
    1.G[(CW20)]
    1.H[Recipient]

    1.A ---> |"(1) set_rewards_updater"| 1.C
    1.F ---> |"(2.1) create_rewards_submission"| 1.B
    1.G ---> |"(2.2) CW20::TransferFrom"| 1.B
    1.C ---> |"(3) submit_root"| 1.B
    1.D ---> |"(4.1) set_claimer_for"| 1.E
    1.E ---> |"(4.2) process_claim"| 1.B
    1.G ---> |"(4.3) CW20::Transfer"|1.H
    end
```
