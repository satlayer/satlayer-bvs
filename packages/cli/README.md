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
