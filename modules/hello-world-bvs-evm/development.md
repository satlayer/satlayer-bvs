# hello-world-bvs-evm development docs

## Prerequisites

- Activate the above accounts, so transfer some native token to these accounts.
- [Golang](https://golang.org/dl/)
- [Redis](https://redis.io/download) (required for running the aggregator)

## Contracts And Account Register

### 0. Prepare Environment

- Download the satlayer cli from [satlayer-cli repo](https://github.com/satlayer/satlayer-cli). And run the readme to build and install.
- Some official contract address can be found in the [official contract readme doc](https://github.com/satlayer/satlayer-core).

Before starting, ensure that you have the following tools installed on your development machine:

- Use the `satlayer-cli keys evm-create-account <password>` command to Generate some accounts. Some accounts display in the below example.
  - Account `caller` to run the task.
  - Account `operator` to run the offchain node
  - Account `aggregator` to run the aggregator node
- Then copy the account file to `~/.eth/keystore` directory.

### 1: deploy the bvs-squaring-evm contract

- Deploy the bvs-squaring contract on evm testnet.
- Get the deployed contract address as bvsAddress.

### 2: register the bvs contract to directory manager

```shell
# everyone can register bvs, so everyone account can register bvs
satlayer directory reg-bvs <userKeyName> <bvsAddress> <bvsChainName> <bvsChainID>
```

After run the above command, will output the bvsHash string. The BVSHash will be used in the following **hello-world-bvs-evm Run** section.

## hello-world-bvs-evm Run

The BVS program follows a structured flow where tasks are initiated, processed off-chain, aggregated, and finally rewarded. The following steps outline the complete process for running the demo.

## Run Steps

To set up and run the demo, follow these steps:

### 0. Prepare Environment

- Ensure you have a running Redis server.

### 1. Run TaskMonitor

- The TaskMonitor continuously tracks and updates the status of ongoing BVS tasks:
- Modify the `env.toml` file located in the `task` directory under the `[owner]` section to match your local machine and bvsHash.

- just run

```bash
cd task
go run main.go monitor
```

- build run

```bash
cd task
go build -o task-cli .
./task-cli monitor
```

### 2. Run Aggregator

- The Aggregator collects and pushes the processed results, ensuring they are available for further use.
- Modify the `env.toml` file located in the `aggregator` directory if you want to use a different database, host port or account.

  - `[app]` section to set the aggregator host and port.
  - `[database]` section to match your Redis server configuration.
  - `[owner]` section to match your account in local machine.

- just run

```bash
cd aggregator
go run main.go
```

- build run

```bash
cd aggregator
go build -o aggregator-cli .
./aggregator-cli
```

If you want to run more than one aggregator, please modify the `env.toml` file, and then, in new terminal run the above commands.

### 3. Run OffchainNode

- The Offchain Node performs the core BVS computations off-chain, ensuring results are processed securely and efficiently:
- Modify the `env.toml` file located in the `bvs_offchain` directory under the `[owner]` section to match your local machine and `[aggregator]` section to match your aggregator node.

- just run

```bash
cd offchain
go run main.go
```

- build run

```bash
cd offchain
go build -o offchain-cli .
./offchain-cli
```

### 4. Run TaskCaller

The TaskCaller sends new BVS tasks to the system and begins the monitoring process:

- just run

```bash
cd task
go run main.go caller
```

- build run

```bash
cd task
go build -o task-cli .
./task-cli caller
```
