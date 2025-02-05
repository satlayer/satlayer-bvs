# SatLayer BVS

## Getting Started & Development Guide

> [!TIP]  
> We use JetBrains IDEs (IntelliJ IDEA) as our primary IDE.
> The `.idea/` and `*.iml` is included in git to make it easier for new developers to get started.

This is a monorepo with Rust, Go, and JavaScript projects.
You need to install the following tools to get started:

1. Install Node (NVM): `https://github.com/nvm-sh/nvm?tab=readme-ov-file#install--update-script`
2. Install Rust: `https://www.rust-lang.org/tools/install`
3. Install Go: `https://go.dev/doc/install`

After installing the tools, you need to install `pnpm` and `turbo` globally as well as the dependencies of the project:

### Install Pnpm, Turbo, and Dependencies

```sh
corepack enable pnpm
pnpm setup
pnpm install turbo --global
pnpm install
```

### Project Layout

```txt
├── crates/                   Rust
│   ├── cw-*                  <- CosmWasm contracts
│   └── *                     <- Other rust crates
├── modules/                  Go
├── packages/                 <- JavaScript, Solidity, etc.
├── README.md
└── turbo.json                <- Turbo configuration (monorepo task runner)
```

### Why Monorepo?

### Why use Turbo, PNPM for a Rust/Go project? 