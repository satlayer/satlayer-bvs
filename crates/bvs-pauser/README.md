# BVS Pauser

Allowing one centralized contract to manage the operational state of the ecosystem.

Two main functions are provided by this contract:

```rust
pub enum QueryMsg {
    #[returns(IsPausedResponse)]
    IsPaused {
        /// The (contract: Addr) calling this
        #[serde(rename = "c")]
        contract: String,
        /// The (method: ExecuteMsg) to check if it is paused
        #[serde(rename = "m")]
        method: String,
    },

    #[returns(CanExecuteResponse)]
    CanExecute {
        /// The (contract: Addr) calling this
        #[serde(rename = "c")]
        contract: String,
        /// The (sender: Addr) of the message
        #[serde(rename = "s")]
        sender: String,
        /// The (method: ExecuteMsg) to check if it is paused
        #[serde(rename = "m")]
        method: String,
    },
}
```

For downstream contracts,
you don't have to manually implement query functions to check if a contract is paused or if a sender can execute a method.
During the instantiation of your contract,
you can call the `set_pauser` function to set the address of the pauser contract.

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(/*...*/) -> Result<Response, ContractError> {
    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;
}
```

And then, in your execute function,
you can call the `assert_can_execute` function to check if the sender can execute the method.

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(/*...*/) -> Result<Response, ContractError> {
    bvs_pauser::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;
}
```
