---
sidebarTitle: Governance as a BVS
---

# [Governance as a BVS Example](https://github.com/satlayer/satlayer-bvs/blob/main/examples/cw/governance/README.md)

## Overview

This example demonstrates using a social committee as the governing body for
Bitcoin Validated Service (BVS) integrated with SatLayer (shared security provider).
The service for this example is simply a solidity based multi-signature contract,
a fixed membership committee that can be used to govern the BVS, for the following:

- Slashing lifecycle
  - Slashing Request
  - Locking the collateral as part of the slashing lifecycle
  - And finalizing the slashing request - moving the slashed collateral to the BVS contract balance.

- Reward distribution
  - This example imagines a scenario where there is a reward trigger mechanism off-chain that triggers
    and injects rewards into the SLAY reward contract through the BVS contract.
  - The committee is responsible for reviewing, proposing, voting, and executing the reward distribution.
  - For simplicity, this example assumes that operators are also node runners.
    Rewards are paid out as part of their sovereign native chain mechanisms to their operators separately.
    The node runners (operators) additionally act as vault curators on BVS,
    such that the rewards are only paid to the retail stakers that stake into the vaults.
  - The example also assumes that the reward node runners are required to curate vaults on BVS.
  - Operators are free to stake to their own delegated vaults to earn additional rewards.

## Getting started

Have a docker environment set up on the machine and follow these steps to run the example:

1. Run `turbo run build` at the root of this monorepo to build the test containers this example uses.
2. Run `pnpm install` in the `examples/evm/governance` (the root of this project) directory to install dependencies.
3. Run `pnpm run build` to build the Solidity contract.
4. Run `pnpm run test` to start the off-chain service tests.
5. Optionally run `pnpm run test:sol` to run solidity specific tests.

## Disclaimer

This example in particular is designed to serve as supplementary material for BVS developers looking to integrate with SatLayer.
Such that the social committee, as a governing body, is purely an arbitrary pick for this example, for simplicity.
The example does not attempt to suggest any reward/slash strategy but rather gets developers familiar with BVS to SatLayer integration.
