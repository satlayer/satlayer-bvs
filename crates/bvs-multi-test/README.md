# BVS Multi Test

BVS Multi Test is a testing utility for the SatLayer ecosystem.
It provides a convenient way to bootstrap all the necessary contracts for testing services in the BVS ecosystem.

This crate is designed
to simplify the testing process for BVS ecosystem contracts
by providing a pre-configured environment with all the necessary contracts already initialized.
It's similar to [cw-multi-test](https://github.com/CosmWasm/cw-multi-test),
but specifically tailored for the BVS ecosystem.

> This crate is only compiled for non-wasm32 targets
> (i.e., for testing purposes) and is not included in the final Wasm binary.

## Features

- Initializes core BVS contracts (Pauser, Registry, VaultRouter) with a single function call
- Provides helper methods to deploy and configure various vault types:
  - Bank vaults (for native tokens)
  - CW20 vaults (for CW20 tokens)
- Simplifies the creation of CW20 tokens for testing purposes

## Usage

BVS Multi Test integrates with the following BVS contracts:

- `bvs-pauser`: Contract for pausing functionality
- `bvs-registry`: Contract for registry functionality
- `bvs-vault-router`: Contract for routing between different vaults
- `bvs-vault-bank`: Contract for bank vaults (native tokens)
- `bvs-vault-cw20`: Contract for CW20 vaults (CW20 tokens)

When you create a new `BvsMultiTest` instance, it automatically initializes these core contracts.
You can then use the provided methods to deploy and configure additional contracts as needed.

### Basic Setup

```rust
use bvs_multi_test::BvsMultiTest;
use cosmwasm_std::testing::mock_env;
use cw_multi_test::App;

#[test]
fn new() {
    let mut app = App::default();
    let env = mock_env();
    BvsMultiTest::new(&mut app, &env);
}
```

### Deploying a Bank Vault

```rust
#[test]
fn deploy_bank_vault() {
    let mut app = App::default();
    let env = mock_env();
    let bvs = BvsMultiTest::new(&mut app, &env);

    let operator = app.api().addr_make("operator");
    let denom = "baby".to_string();

    let vault = bvs.deploy_bank_vault(&mut app, &env, operator.clone(), denom.clone());

    assert_eq!(vault.init.operator, operator.to_string());
    assert_eq!(vault.init.denom, denom);
}
```

### Deploying a CW20 Token and Vault

```rust
#[test]
fn deploy_cw20_vault() {
    let mut app = App::default();
    let env = mock_env();
    let bvs = BvsMultiTest::new(&mut app, &env);

    let owner = app.api().addr_make("owner").to_string();
    let token = bvs.deploy_cw20_token(&mut app, &env, "APPLE", owner.clone());

    let operator = app.api().addr_make("operator");
    let vault = bvs.deploy_cw20_vault(&mut app, &env, operator.clone(), token.addr.clone());

    assert_eq!(vault.init.operator, operator.to_string());
    assert_eq!(vault.init.cw20_contract, token.addr.to_string());
}
```
