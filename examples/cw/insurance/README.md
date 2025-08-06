---
sidebarTitle: Insurance as a BVS
---

# Insurance as a BVS Example

This example demonstrates how to use the Bitcoin Validated Service (BVS) to provide insurance services.
The service implements a complete insurance policy lifecycle with automated underwriting, claims processing,
and rewards distribution.

## Overview

In this example, operators serve as vault curators rather than node runners.
They manage vaults and provide sufficient collateral for insurance services without needing to produce results — in other words, no node operations are required.

Both operators and stakers receive rewards from a portion of insurance premiums paid by insurees (e.g. BVS users).
The system comprehensively manages the entire policy lifecycle,
handling everything from initial purchase through claims processing to final payouts.

## Key Assumptions

This example operates with a simplified model where all capital comes from a single `Operator` managing a single `Vault`, and is meant to show how insurance providers can tap onto the Bitcoin collateral provided through the SatLayer restaking protocol.

The system allows 80% of the vault balance to be utilized for policy issuance and claims processing.
Only one operator is registered to the service, and this operator is responsible for vault management.

Policies have a standard duration of 1 year with premiums fixed at 2% of the coverage amount.
All policy premiums must be paid upfront for the entire policy duration.

Insurees can submit claims (to the service) at any time during the policy period.
When claims are approved, they pay out 100% of the coverage amount.

The payout and rewards are paid out in the same token as the vault's staked asset.

The reward structure allocates 50% of total premiums collected from active policies to be distributed as rewards.
These rewards are shared between operators (10%) and stakers (90%), with distribution proportional to staked amounts.

## Getting started

Prerequisites for this example:

1. Install Rust and Cargo: https://www.rust-lang.org/tools/install
2. Basic understanding of [CosmWasm contracts](/cosmwasm/developer)
3. Node.js for development of the off-chain service: https://nodejs.org/en/download/
4. Docker to run off-chain node: https://docs.docker.com/get-docker/

Project structure:

```txt
insurance/
├── service/
│   ├── api.ts                  <- Gateway for on-chain contract communications
│   ├── policy-manager.ts       <- Core policy management functionality
│   ├── policy-manager.test.ts  <- Lifecycle example (policy purchase, claims, payouts)
└── README.md
```

### How to run

Clone and copy contents of `./examples/insurance` [from GitHub](https://github.com/satlayer/satlayer-bvs/tree/main/examples/insurance) to your local machine.
Install with `npm install` and `npm run test` to set up the environment.

```sh filename="run.sh"
npm run install
npm run test
```

### How to build the contract

To build the CosmWasm contract, you will need to have Docker running, and then you can run the following command:

```sh filename="build.sh"
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```

## Core Components

### PolicyManager

The `PolicyManager` is an off-chain service
that manages insurance policies with automated underwriting and claims verification.
It interacts with SatLayer core on-chain contracts to handle the entire policy lifecycle.

The service handles all aspects of policy lifecycle management,
from initial purchase through claims processing to final payouts.
It also manages rewards calculation and distribution to earners,
ensuring operators and stakers receive their proportional share of premiums collected.

### API Service

The API service acts as a gateway to on-chain contract communications.
It provides methods to interact with several SatLayer core contracts.
Through this service, the system can access the Vault Bank for managing staked assets,
the Router for handling slashing requests, and the Rewards contract for distributing rewards to stakers.
This abstraction layer simplifies interactions with the blockchain.

## Policy Lifecycle

**Policy Purchase**. The lifecycle begins when an insuree buys a policy with a specified coverage amount.
The system validates the insuree's eligibility and checks
if the vault has sufficient capacity to cover the potential claim.
The premium is then calculated at 2% of the coverage amount.

**Claims Processing**. When an insuree submits a claim, the system verifies its validity through automated processes.
If the claim is verified as legitimate,
the system triggers a slashing of the vault to secure the funds needed for the payout.

**Payout Processing**. After securing the funds and the guardrail voting passes, the system finalizes the slashing request.
The payout amount, which equals 100% of the coverage amount, is then transferred directly to the insuree's account.

**Rewards Distribution**. Throughout this process, the system calculates rewards based on premiums collected from all active policies.
These rewards are distributed between operators (10%) and stakers (90%),
with each participant's share proportional to their staked amount.
