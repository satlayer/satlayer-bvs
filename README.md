# SatLayer BVS

## Getting Started

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Project Layout & Development Guide

```txt
├── crates/                   Rust
│   ├── cw-*                  < CosmWasm contracts            
│   └── *                     < Other rust crates
├── modules/                  Go
├── packages/                 JavaScript, Solidity, etc.
```