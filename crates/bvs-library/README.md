# BVS Library

A standalone library for the BVS project that provides shared functionality for BVS Contracts.

### `bvs_library::*`

Used in other CosmWasm contracts to share common functionality.

- `bvs_library::ownership`: Similar to `Ownable.sol` to assert that the caller is the owner of the contract.
- `bvs_library::addr`: Additional utilities for working with addresses.

### `bvs_library::testing`

Test utilities for BVS contracts for unit and integration tests.
Only compile in non wasm32 targets `#![cfg(not(target_arch = "wasm32"))]`, purely for testing.
Won't be included in the `*.wasm` artifact.

`TestingContract` is a trait that provides a common testing interface for setting up testing contracts.
It provides utils for common functions or traits for easy testing—allowing for testing code to be **colocated at
the cosmwasm** crates, as they should—for integration testing productivity sake.

```rust
pub trait TestingContract<IM, EM, QM> {
    fn wrapper() -> Box<dyn Contract<Empty>>;
    fn default_init(app: &mut App, env: &Env) -> IM;
    fn new(app: &mut App, env: &Env, msg: Option<IM>) -> Self;
}
```

```txt
crates/
├── bvs-library/              <-- You are here
├── bvs-registry/
│   └── src/
│       └── testing.rs        <-- put here with `#![cfg(not(target_arch = "wasm32"))]`
├── bvs-rewards-coordinator/
│   └── src/
│       └── testing.rs        <-- put here with `#![cfg(not(target_arch = "wasm32"))]`
```

For integration tests, downstream crates can implement `TestingContract` for execute/query/init/defaults functionality.

```rust
pub struct RegistryContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RegistryContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        // You can get a contract address by their label, allowing you to orchestrate "instinctively" — however, we still need to remove circular dependency.
        // Self::get_contract_addr(app, "contract-label")
        InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
            initial_paused: false,
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "contract-label", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}
```
