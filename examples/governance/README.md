# Overview

This example demonstrates using social committee as governing body for BVS (bitcoin validated service) integrated with Satlayer (shared security provider).
The service for this example is simply a CW3, fixed membership committee that can be used to govern the BVS, regarding the following:

- Slashing lifecycle

  - Slashing Request
  - Locking the collateral as part of slashing lifecycle
  - And finalizing the slashing request - moving the slashed collateral to the BVS contract balance.

- Reward distribution
  - This example imagine a scenario where there is a reward triggers mechanisms off-chain that triggers and injects rewards to Satlayer reward contract
    through the BVS contract.
  - The committee is responsible for reviewing, proposing voting and executing the reward distribution.
  - For the simplicity, this example assume that operators are also node runners, reward are paid out as part of their sovereign native chain mechanisms to their operator separately. The node runners (operator) additionally acts as vault curators on BVS. Such that the reward are only paid to the retail stakers that stake into the vaults.
  - The example also assumes that the reward node runner are required to curate vaults on BVS.
  - Operator are free to stake to their own delegated vaults to earn additional rewards.

# Disclaimer

This example in particular is designed to serve as supplementary material for BVS developers looking to integrate with Satlayer. Such that the social committee as governing body is purely an arbitrary pick for this example for the simplicity. The example does not make in any attempt to suggest any one reward / slash strategy but rather get developers familiar with BVS to Satlayer integration.

# Directory Guide

```bash
.
├── contract                            <-- Contains the governance contract code
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── generate.mjs
│   ├── governance-contract.d.ts
│   ├── package.json
│   ├── src
│   │   ├── bin
│   │   │   └── schema.rs
│   │   ├── contract.rs
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   ├── msg.rs
│   │   ├── state.rs
│   │   └── testing.rs
│   └── tests
│       └── integration_test.rs
├── README.md                           <-- This file
└── service
    ├── package.json
    └── src
        ├── api.ts                      <-- API for interacting with Satlayer protocol contracts.
        ├── reward.test.ts              <-- Reward lifecycle example implemented as Tests for reward distribution functionality
        ├── service.ts                  <-- Contains the off-chain reward trigger emulation codes.
        └── slash.test.ts               <-- Slashing lifecycle example implemented as Tests for slashing flow on Satlayer.
```
