# SatLayer CLI

SatLayer CLI is a multi-platform command-line tool built with Go that allows developers and users to interact with the SatLayer smart contracts.
It provides commands for managing various contracts and reward distributions.
The CLI package automatically detects your operating system and architecture during installation and sets up the appropriate binary.

## Installation

You can install the SatLayer CLI globally using npm.
This will allow you to use the `satlayer` command from anywhere in your terminal.

```bash
npm install @satlayer/cli --global

satlayer --help
```

### Using it in node.js

```json
{
  "name": "project-name",
  "scripts": {
    "help": "satlayer --help"
  },
  "dependencies": {
    "@satlayer/cli": "latest"
  }
}
```

### Development

This `@satlayer/cli` package is the front to the actual CLI,
which is written in Go and compiled to a binary for each platform.
The Go code is located in the `modules/coswasm-cli` directory,
`package.js` prepares the binaries for each platform and architecture and
uses `optionalDependencies` to install the correct binary for the current platform.

You can build the CLI for all platforms and architectures by running:

```shell
turbo run build
```
