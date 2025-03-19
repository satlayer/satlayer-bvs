use crate::error::ContractError;
use crate::state::SLASHER;
use cosmwasm_std::{Addr, Deps, MessageInfo};

fn assert_slasher(deps: Deps, info: MessageInfo) -> Result<Addr, ContractError> {
    let slasher = SLASHER.load(deps.storage)?;
    if slasher != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(slasher)
}
