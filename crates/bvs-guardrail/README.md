# BVS Guardrail

BVS Guardrail is a smart contract that serves as a final check for the slashing request before it can be finalized.

## Overview

BVS Guardrail implements a voting system based on the [CW3 multisig specification](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw3/README.md) and [CW4 group membership](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw4/README.md).
It ensures
that slashing requests are only finalized
if they have been approved by a sufficient number of eligible voters within the specified voting period.

This contract is largely adapted from [cw3-flex-multisig](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw3-flex-multisig) and [cw4-group](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw4-group) implementations.

## Usage

The contract is initialized with a set of members (voters) and a threshold configuration that determines how many votes are required for a proposal to pass. Once initialized, authorized members can create and vote on slashing proposals.

Proposals are created when a slashing request is initiated.
Members can vote on these proposals within a specified voting period.
If the proposal receives enough votes before the voting period ends, it is considered approved and the slashing request can be finalized.
