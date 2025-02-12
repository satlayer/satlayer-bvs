# satlayer-cli

## Introduction

satlayer-cli is a command line tool for Satlayer.With this command-line tool, users can register BVS, register operators, stake amounts, claim rewards, and perform queries, among other operations.

Mainly includes the following module commands:

- `directory` : This module is responsible for managing and maintaining directory information within the BVS system, including the registration, updating, and querying of BVS and Operators.
- `delegation`: This module manages delegation operations, allowing users to delegate their stakes to other operators to enhance the network's security and efficiency.
- `strategy`: This module is used to formulate and manage various strategies to optimize the staking and reward distribution processes, ensuring that users can maximize their returns.
- `strategy-base`: This module provides the definition and implementation of base strategies, allowing users to adjust and extend them according to their specific needs.
- `reward`: This module is responsible for managing the distribution and claiming of rewards, ensuring that users receive their rewards promptly after participating in staking and other activities.
- `chain`: This module handles input and output operations related to the blockchain, ensuring accurate data transmission and processing to support the efficient operation of the entire system.
- `slash`: This module providers interaction with Slash contract.

## Install

To build and install, run this command:

```shell
make install
```

## Config

- Default config file in "{$HOME}/.config/satlayer/config.toml"
- If config file is not found, it will be created automatically
- If you want to use other config file, please set SATLAYER_CONFIG environment variable

## Run

- Run `satlayer-cli --help` to get help

## Run Test

- Run `make test`
