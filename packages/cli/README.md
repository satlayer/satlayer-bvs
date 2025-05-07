# SatLayer CLI

SatLayer CLI is a multi-platform command-line tool built with Go that allows developers and users to interact with the SatLayer smart contracts.
It provides commands for managing various contracts and reward distributions.
The CLI package automatically detects your operating system and architecture during installation and sets up the appropriate binary.

## Installation

```bash
npm install @satlayer/cli
```

<details>
<summary>Build from Source</summary>

1. Clone the repository
2. Install dependencies: `pnpm install`
3. Build the CLI: `pnpm build`

The source code is organized as follows:

- **Source Code**: Located in `modules/cosmwasm-cli`
- **Platform-specific Packages**: Located in `packages/cli/cli-{os}-{arch}`
- **NPM Package Wrapper**: Located in `packages/cli`

</details>
