# SatLayer BVS

## Getting Started

> [!TIP]  
> We use JetBrains IDEs (IntelliJ IDEA) as our primary IDE.
> The `.idea/` and `*.iml` is included in git to make it easier for new developers to get started. 

This is a monorepo with Rust, Go, and JavaScript projects. 
You need to install the following tools to get started: 

1. Install Node (NVM): `https://github.com/nvm-sh/nvm?tab=readme-ov-file#install--update-script`
   1. Enable PNPM via corepack: `corepack enable pnpm`
2. Install Rust: `https://www.rust-lang.org/tools/install`
3. Install Go: `https://go.dev/doc/install`

## Project Layout & Development Guide

```txt
├── crates/                   Rust
│   ├── cw-*                  < CosmWasm contracts            
│   └── *                     < Other rust crates
├── modules/                  Go
├── packages/                 JavaScript, Solidity, etc.
```