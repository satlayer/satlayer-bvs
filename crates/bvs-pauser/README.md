# BVS Pauser

The BVS Pauser contract is a specialized smart contract
designed to provide a centralized pause mechanism for the entire ecosystem.
It allows an authorized owner to pause and unpause contract functionality across the system,
which is crucial for emergency response to security incidents, bugs, or other critical issues.
Allowing for quick and efficient responses to any issues that may arise.

The contract provides these main query functions:

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

### Integration with Other Contracts

The BVS Pauser contract is designed to be easily integrated with other contracts in the ecosystem.
For downstream contracts,
you don't have
to manually implement query functions to check if a contract is paused or if a sender can execute a method.
The BVS Pauser provides a simple API for integration:

1. During the instantiation of your contract, set the address of the pauser contract:

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(/*...*/) -> Result<Response, ContractError> {
    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;
}
```

2. In your execute function, check if the operation is allowed before proceeding:

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(/*...*/) -> Result<Response, ContractError> {
    bvs_pauser::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    // Continue with execution if not paused
}
```

This simple integration ensures that all BVS contracts respect the global pause state,
securing and manageable ecosystem.
