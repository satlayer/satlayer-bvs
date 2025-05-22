# Insurance Service Example

This example demonstrates how to use the BVS (Bitcoin Validated System) to provide insurance services.
In this example, operator are not required to run the node or produce results.
The operator acts as a vault curator
and is responsible for managing the vault and providing sufficient collateral for the insurance service.

The operator and staker will be rewarded with part of the insurance premium that is paid by the BVS users (insuree).

## Project Structure

```txt
insurance/
├── service/
│   ├── contract/    <- CosmWasm contract managed by the service and the address is registered in SatLayer Registry as a service
│   └── offchain/    <- Off-chain component run by the service
└── README.md
```
