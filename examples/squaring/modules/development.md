# hello-world-bvs development docs

## Prerequisites

Before starting, ensure that you have the following tools installed on your development machine:

- Install `babylond` cli tool. Please refer to [babylond](https://github.com/babylonlabs-io/babylon)
- Use the `babylond keys add <keyName>` command to Generate some accounts. Some accounts display in the below example.
  - Account `caller` to run the task.
  - Account `operator` to run the offchain node
  - Account `aggregator` to run the aggregator node
  - Account `monitor` to run the monitor node

> Note: babylond keyring backend types:
>
> - os: Use the operating system's secure storage (e.g. keychain for macOS, libsecret for Linux). This is the default.
> - file: Use the file system to store keys
> - test: Use memory to store keys, mainly for testing purposes
>
> when use the key, need to set the `keyringBackend`.

- Activate the above accounts, so transfer some native token to these accounts.
- [Golang](https://golang.org/dl/)
- [Redis](https://redis.io/download) (required for running the aggregator)

## Contracts And Account Register

### 0. Prepare Environment

- Download the satlayer cli from [satlayer-cli repo](https://github.com/satlayer/satlayer-cli). And run the readme to build and install.
- Some official contract address can be found in the [official contract readme doc](https://github.com/satlayer/satlayer-core).

### 1: deploy the bvs-squaring contract

- Read the [bvs-squaring contract readme doc](./contract/bvs-squaring/README.md).
- Deploy the bvs-squaring contract on Babylon testnet.
- Get the deployed contract address as bvsAddress.

### 2: register the bvs contract to directory manager

```shell
# everyone can register bvs, so everyone account can register bvs
satlayer directory reg-bvs <userKeyName> <bvsAddress>
```

After run the above command, will output the bvsHash string. The BVSHash will be used in the following **hello-world-bvs Run** section.

### 3: register the operators

```shell
# register operator to delegate manager
# the approverAddress is the witness of the operator
# the operatorAddress not be the operator account、the staker account
# If operator does not want a approver, you can set the approverAddress as 0
satlayer delegation reg-operator <operatorKeyName> <approverAddress>

# register operator to directory manager
satlayer directory reg-operator <operatorKeyName>
```

> Tips: If you don't want to run reward, you can skip these steps.

### 4: register the strategy

This step commands only strategy manager can do.

```shell
# set the delegation manager
satlayer strategy  set-delegation-manager  <userKeyName> <delegationManagerAddress>
# set the strategy whitelist
satlayer strategy  set-strategy-whitelist  <userKeyName> <strategyWhitelistAddress>
```

### 5. staker delegate to operator

This step commands need staker account.

```shell
# delegate staker to operator
# This approver key is the one used when the operator was registered
# If the operator approver address is 0, you can skip the approverKeyName
satlayer delegation delegate-to <stakerKeyName> <operatorAddress> [approverKeyName]

# increase token allowance to delegation manager
satlayer chain increase-token-allowance <stakerKeyName> <cw20TokenAddress> <delegationManagerAddress> <amount>

# deposit into strategy
satlayer strategy deposit-strategy <stakerKeyName> <strategyAddress> <cw20TokenAddress> <amount>
```

### 6. Reward uploader

this step commands need uploader account.

- First, the reward coordinator contract owner need set the uploader address to be submitter address.
- Second, keep the uploader account have enough cw20 token balance to upload reward.
- Third, uploader need call `IncreaseTokenAllowance` to approve the reward Coordinator contract transfer Cw20 token.

```shell
# increase token allowance to reward coordinator
satlayer chain increase-token-allowance <uploaderKeyName> <cw20TokenAddress> <rewardCoordinatorAddress> <amount>
```

## hello-world-bvs Run

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
