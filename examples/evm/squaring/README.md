# Computational Squaring Example

This example demonstrates how to use off-chain computation with on-chain verification through a
[service](../../../getting-started/services/page.mdx). It introduces a simple squaring function (`n^2`) to illustrate the
core mechanics of SatLayer’s BVS model. Although squaring a number is a trivial task, this pattern
generalizes to more complex use cases involving cryptographic proofs, simulations, or data aggregation.

The key idea is to offload heavy computation from the blockchain, while still enabling trustless, verifiable
results that can trigger slashing if misbehavior occurs.

## Lifecycle Overview

The example follows a simple lifecycle:

- Requests are sent to the contract without the need for heavy on-chain computation.
- The off-chain service,
  ran by the operators, computes the square of a given number and submits the result to the contract.
- Slashing is triggered if the operator submits an incorrect result.

This approach enables computational efficiency while maintaining crypto-economic accountability.

## Getting started

Have a docker environment set up on the machine and follow these steps to run the example:

1. Run `turbo run build` at the root of this monorepo to build the test containers this example uses.
2. Run `pnpm install` in the `examples/evm/squaring` (the root of this project) directory to install dependencies.
3. Run `pnpm run build:sol` to build the Solidity contract.
4. Run `pnpm run test` to start the off-chain service tests.
5. Optionally run `pnpm run test:sol` to run solidity specific tests.

## Project overview

This project demonstrates a Bitcoin Validated Service (BVS) using a simple squaring function as an example.
It consists of two main components: the on-chain contract, and the off-chain service.

We use squaring as a simple example to demonstrate the concept of off-chain computation with on-chain verification.
Squaring is `n^2`, where `n` is the input number, and the result is the square of that number.

While this example is simple and not meant for production,
it illustrates the principles of off-chain computation and on-chain verification.

The general request respond lifecycle is as follows:

1. Users submit computation requests to the contract
2. Operators (running the off-chain service) detect these requests
3. Operators perform the computation off-chain
4. Operators submit the results back to the contract
5. The contract verifies the results and slashes operators who submit incorrect answers

### On-chain Contract (Solidity)

Located in the `src/BVS.sol` directory, this Rust-based smart contract:

- Accepts computation requests from users
- Verifies responses from operators to prove fault
- Manages operator registration and slashing mechanisms
- Provides query endpoints for retrieving computation results

### Off-chain Service (TypeScript)

Located in the `src/*.ts` directory, this Node.js service:

- Continuously monitors the blockchain for new computation requests
- Performs the actual computation (i.e. squaring numbers) off-chain
- Submits results back to the on-chain contract
