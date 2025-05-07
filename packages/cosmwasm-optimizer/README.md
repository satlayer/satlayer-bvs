# SatLayer CosmWasm Optimizer

A command-line tool that simplifies the process of optimizing CosmWasm smart contracts for deployment.
This tool wraps the official CosmWasm optimizer Docker image,
providing an easy-to-use interface for building production-ready WebAssembly (Wasm) binaries.

## Features

- Optimizes CosmWasm contracts for minimal size and gas usage
- Generates contract schema files automatically
- Supports both AMD64 and ARM64 architectures
- Provides Docker build caching for faster builds
- Outputs optimized contract as `contract.wasm` with checksums

## Installation

```bash
npm install -g @satlayer/cosmwasm-optimizer
```

## Usage

Basic usage (run in your contract directory):

```bash
cosmwasm-optimizer
```

This will:

1. Build your contract using the CosmWasm optimizer Docker image
2. Generate schema files
3. Output the optimized contract to `./dist/contract.wasm`

### Options

- `--root <path>`: Specify a different root directory for the contract (default: current directory)
- `--name <name>`: Specify a name for the contract (default: directory name)

Example with options:

```bash
cosmwasm-optimizer --root ./my-contract --name my-awesome-contract
```

### Docker Caching

You can enable Docker build caching by setting the following environment variables:

- `DOCKER_CACHE_FROM`: Registry to pull cache from
- `DOCKER_CACHE_TO`: Registry to push cache to

Example:

```bash
export DOCKER_CACHE_FROM=my-registry.com/cosmwasm-cache
export DOCKER_CACHE_TO=my-registry.com/cosmwasm-cache
cosmwasm-optimizer
```

## Integration with npm scripts

Add to your `package.json`:

```json
{
  "scripts": {
    "build": "cosmwasm-optimizer"
  }
}
```

Then run:

```bash
npm run build
```

## Output

After running the optimizer, you'll find the following files in the `./dist` directory:

- `contract.wasm`: The optimized WebAssembly binary
- `checksums.txt`: SHA256 checksums of the binary
- `schema/`: Directory containing JSON schema files
- `schema.json`: Combined schema file

## License

MIT
