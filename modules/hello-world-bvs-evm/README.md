# SatLayer Squaring BVS EVM Demo

## Overview

The **SatLayer Squaring BVS EVM Demo** is a comprehensive demonstration project designed to showcase the integration and operation of the SatLayer API with the Squaring BVS (Actively Validated Services). The project demonstrates how various components within the SatLayer ecosystem interact to provide a robust solution for managing decentralized services. The demo is structured into multiple directories, each dedicated to a specific function, allowing for modular development, testing, and deployment. This readme will guide you through the setup, building, and running of the demo, ensuring that you can easily replicate the environment on your local machine.

## Project Structure

The demo is divided into several components, each responsible for a specific aspect of the Squaring BVS process. Below is a high-level architecture and the relevant components:

The BVS Program consists of the following key components:

- **Task**: Sends BVS tasks and monitors their results. (caller & monitor)
- **Offchain Node**: Executes BVS logic off-chain, performing the necessary computations.
- **Aggregator**: Collects and pushes task results for further processing.

## Run test

```bash
pnpm test
```

## Development

To run the demo, please follow the steps in the [development docs](./development.md).

## Conclusion

This demo provides a complete overview of how to implement and run a SatLayer Squaring BVS service using the SatLayer API. The modular architecture and clear run steps ensure that you can replicate the environment easily, allowing you to explore the various capabilities of the SatLayer ecosystem.
