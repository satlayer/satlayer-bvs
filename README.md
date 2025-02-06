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
4. Install Docker: `https://docs.docker.com/get-started/get-docker/`

After installing the tools, you need to install `pnpm` and `turbo` globally as well as the dependencies of the project:

<details>
<summary>Recommended Docker Settings:</summary>

**Docker Engine:**

- `builder.gc.defaultKeepStorage`: `200GB`
- `default-address-pools[0]`: `{"base": "10.32.0.0/12", "size": 26}`

Due to heavy usage of Docker,
we recommend a higher defaultKeepStorage setting to avoid running out of disk space too quickly.
And a bigger address pool to have more IP addresses available for Docker containers.

**Resource Allocation:**

- `Memory`: Half of the total memory, ideally above 16GB
- `CPU`: More than 80% of available cores, ideally 7 or more

</details>

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

Proper separation of concerns is about grouping related functionality in ways
that mirror how the software actually evolves,
rather than defaulting to arbitrary technical boundaries (e.g., splitting everything by file type).
No matter if our code is in Go, Rust, Solidity, or WASM, the central idea remains:
separating concerns should make our code easier to navigate, understand,
and maintain.

In practice, this means organizing functionality by features or responsibilities instead of just the code type.
For instance, a feature that touches multiple languages or modules should be treated as a single “concern,”
so that related logic is in one place and not scattered across repos.

### Why use Turbo, PNPM for a Rust/Go project?

Although the core of this monorepo is Rust and Go, we’re ultimately exporting libraries,
SDKs, and user-facing code that often revolves around JavaScript.
Adopting a JavaScript-centric toolchain like Turbo and PNPM offers a simpler,
more popular, and faster alternative to Bazel-like systems.
Turbo is feature-rich yet straightforward to configure,
prioritizing convention over hermetic complexity and letting each language
(Cargo for Rust, Go modules for Go, PNPM for JS) handle its own dependencies.
This means a Go developer doesn’t need to manage Rust builds,
and a JavaScript developer doesn’t have to worry about Cargo.
As long as the necessary dependencies are installed, Turbo just works.
Additionally, while Rust/Go monorepos aren’t as widely supported,
combining them with Turbo and PNPM bridges the gap and streamlines multi-language development.

> [!WARNING]
> Although Turbo greatly speeds up development by caching tasks based on their inputs and outputs,
> it’s not a magic bullet.
> If you encounter unexpected issues or incorrect build results,
> try running turbo --force to invalidate the cache and rebuild everything from scratch.
> If you spot any errors with our `turbo.json` files, please fix them promptly to avoid further headaches.
