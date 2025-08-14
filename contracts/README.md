# @satlayer/contracts

Artifact bundle for SatLayer EVM smart contracts.
This package ships the compiled Foundry artifacts (ABIs, bytecode, metadata, and interfaces) so you can easily
integrate SatLayer contracts in your dapps, scripts, and SDKs.

## Installation

```bash
npm install @satlayer/contracts
# You will need OpenZeppelin contracts as a peer dependency if you're interacting with ISLAYVault
npm install @openzeppelin/contracts
```

Compiled artifacts for the core SatLayer contracts (ABI, bytecode) are under the `./out` export paths.

```ts
import { abi as SLAYRegistryV2Abi } from "@satlayer/contracts/out/ISLAYRegistryV2.sol/ISLAYRegistryV2.json";
```

Contract interfaces are available in the `./src/interface` directory.

```solidity
import {ISLAYRegistryV2} from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";
```

With `remappings.txt`:

```txt
@satlayer/contracts/=node_modules/@satlayer/contracts/src/
```

## Usage examples

### viem

```ts
import { createPublicClient, http, getContract } from "viem";
import { mainnet } from "viem/chains";
import { abi as SLAYRegistryV2Abi } from "@satlayer/contracts/out/ISLAYRegistryV2.sol/ISLAYRegistryV2.json";

const client = createPublicClient({ chain: mainnet, transport: http() });

const registry = getContract({
  address: "0x...registryAddress", // provide the deployed address
  abi: SLAYRegistryV2Abi,
  client,
});

// example read
const maxForService = await registry.read.maxActiveRelationshipsForService();
```

### ethers.js (v6)

```ts
import { ethers } from "ethers";
import SLAYRegistryV2Artifact from "@satlayer/contracts/out/ISLAYRegistryV2.sol/ISLAYRegistryV2.json" with { type: "json" };

const provider = new ethers.JsonRpcProvider(process.env.RPC_URL!);
const registry = new ethers.Contract("0x...registryAddress", SLAYRegistryV2Artifact.abi, provider);

const maxForService = await registry.maxActiveRelationshipsForService();
```

## Maintainer Guide

### Development

```bash
forge build
forge test
forge snapshot
```

### Dependency management

> We might change how we'll manage dependencies in the future.
> Dependencies are NOT installed as git submodules via `forge install`.
> This repo instead uses `pnpm` to manage dependencies deterministically, moved to `libs/` directory post-installation.
