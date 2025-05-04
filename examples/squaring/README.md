---
sidebarTitle: Computational Squaring
---

# Computational Squaring Example

Off-chain computation with on-chain verification for squaring a number.
Here, we implement a simple squaring function (`n ^ 2`) for demonstration purposes—in a real-world scenario,
you would implement a more complex function.

This example demonstrates how operators perform off-chain computation of squaring a number
with on-chain objectively verifiable results when a fault occurs.

Lifecycle Overview:

- Requests are sent to the contract without the need for heavy on-chain computation.
- The off-chain service,
  ran by the operators, computes the square of a given number and submits the result to the contract.
- Slashing is triggered if the operator submits an incorrect result.

## Getting started

Prerequisites:

1. Rust and Cargo: https://www.rust-lang.org/tools/install
2. Basic understanding of [CosmWasm Contract](/developers/cosmwasm)
3. Docker to run off-chain node: https://docs.docker.com/get-docker/

Project structure:

```txt
squaring/
├── contract/     <- CosmWasm contract
├── service/      <- Off-chain service
├── compose.yml   <- Docker Compose file for running the service
└── README.md     <- This file
```

### How to run

Clone and copy contents of `./examples/squaring` to your local machine.

```shell
docker compose up
```

## Project overview
