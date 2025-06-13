# CosmWasm Schema CLI

A command-line tool for generating type definitions from CosmWasm schema files.

## Overview

CosmWasm Schema CLI is a utility that transforms CosmWasm contract schema files into type definitions for various programming languages. It uses the [quicktype](https://github.com/quicktype/quicktype) library to generate strongly-typed code from JSON schema definitions.

This tool is particularly useful for developers working with CosmWasm smart contracts who need to generate client-side types that match their contract's schema.

## Installation

### Global Installation

```shell script
# Using npm
npm install -g @satlayer/cosmwasm-schema-cli

# Using pnpm
pnpm add -g @satlayer/cosmwasm-schema-cli

# Using yarn
yarn global add @satlayer/cosmwasm-schema-cli
```

### Local Installation

```shell script
# Using npm
npm install --save-dev @satlayer/cosmwasm-schema-cli

# Using pnpm
pnpm add -D @satlayer/cosmwasm-schema-cli

# Using yarn
yarn add -D @satlayer/cosmwasm-schema-cli
```

Add the following script to your `package.json`:

```json
{
  "scripts": {
    "build:schema": "schema-gen ./path/to/schema.json --language typescript --out-dir ./"
  }
}
```

## Usage

### Basic Usage

Generate type definitions for a CosmWasm schema file:

```shell script
schema-gen ./path/to/schema.json
```

By default, this will generate a typescript file in the current directory, using the contract name (or directory name) as the basis for the output filename.

### Options

```
Usage: schema-gen <schema-path> [options]

Options:
  --out-dir, -o    Output directory (default: current directory)
  --language, -l   Target language (default: go)
                   Supported: go, typescript, rust, kotlin, swift, csharp, java, javascript, python, cpp
  --help, -h       Show this help message
```

### Examples

Generate TypeScript types in a specific directory:

```shell script
schema-gen ./contracts/dist/schema.json --language typescript --out-dir ./types
```

Generate Go types:

```shell script
schema-gen ./contracts/dist/schema.json --language go --out-dir ./pkg/types
```

## Supported Languages

- Go
- TypeScript (default)

## How to get `schema.json` file

To generate the `schema.json` file for your CosmWasm contract, you can use `cosmwasm-optimizer` to build your contract.
Read more about it in the [@satlayer/cosmwasm-optimizer](https://github.com/satlayer/satlayer-bvs/tree/main/packages/cosmwasm-optimizer).
