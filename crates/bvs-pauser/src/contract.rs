#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::PAUSED;
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "BVS Pauser";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    PAUSED.save(deps.storage, &msg.initial_paused)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Pause {} => execute::pause(deps, info),
        ExecuteMsg::Unpause {} => execute::unpause(deps, info),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

pub mod execute {
    use super::*;
    use crate::state::PAUSED;

    pub fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        PAUSED.save(deps.storage, &true)?;
        Ok(Response::new()
            .add_attribute("method", "pause")
            .add_attribute("sender", info.sender))
    }

    pub fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        PAUSED.save(deps.storage, &false)?;
        Ok(Response::new()
            .add_attribute("method", "unpause")
            .add_attribute("sender", info.sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsPaused { contract, method } => {
            to_json_binary(&query::is_paused(deps, contract, method)?)
        }
        QueryMsg::CanExecute {
            contract,
            sender,
            method,
        } => to_json_binary(&query::can_execute(deps, contract, sender, method)?),
    }
}

pub mod query {
    use super::*;
    use crate::msg::{CanExecuteResponse, IsPausedResponse, FLAG_CAN_EXECUTE, FLAG_PAUSED};
    use crate::state::PAUSED;

    /// TODO(future): The `_contract` and `_method` are currently not used.
    ///  To implement checking of paused status against contract and method.
    ///  Added for future compatibility, not yet utilized—current design pauses all execute.
    pub fn is_paused(
        deps: Deps,
        _contract: String,
        _method: String,
    ) -> StdResult<IsPausedResponse> {
        let is_paused = PAUSED.load(deps.storage)?;
        Ok(IsPausedResponse::new(is_paused))
    }

    /// TODO(future): The `_contract`, `_sender` and `_method` are currently not used.
    ///  To implement checking of paused status against contract, method and sender.
    ///  Added for future compatibility, not yet utilized—current design pauses all execute.
    pub fn can_execute(
        deps: Deps,
        _contract: String,
        _sender: String,
        _method: String,
    ) -> StdResult<CanExecuteResponse> {
        let is_paused = PAUSED.load(deps.storage)?;
        if is_paused {
            return Ok(CanExecuteResponse::new(FLAG_PAUSED));
        }
        Ok(CanExecuteResponse::new(FLAG_CAN_EXECUTE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::IsPausedResponse;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn instantiate_msg() {
        let mut deps = mock_dependencies();

        let info = message_info(&deps.api.addr_make("sender"), &coins(1000, "rat"));
        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
            initial_paused: false,
        };
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsPaused {
                contract: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let value: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(false, value.is_paused());
    }

    #[test]
    fn pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        let info = message_info(&owner, &[]);
        execute::pause(deps.as_mut(), info).unwrap();

        let contract = deps.api.addr_make("contract").to_string();
        let sender = deps.api.addr_make("sender").to_string();
        let method = "any_method".to_string();

        let response = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert_eq!(true, response.is_paused());

        let response = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        assert_eq!(false, response.can_execute());
    }

    #[test]
    fn pause_unauthorized() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        execute::pause(deps.as_mut(), info).expect_err("Unauthorized");

        let contract = deps.api.addr_make("contract").to_string();
        let sender = deps.api.addr_make("sender").to_string();
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert_eq!(false, res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            assert_eq!(true, res.can_execute());
        }
    }

    #[test]
    fn unpause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &true).unwrap();

        let info = message_info(&owner, &[]);
        execute::unpause(deps.as_mut(), info).unwrap();

        let contract = deps.api.addr_make("anyone").to_string();
        let sender = deps.api.addr_make("sender").to_string();
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert_eq!(false, res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            assert_eq!(true, res.can_execute());
        }
    }

    #[test]
    fn unpause_unauthorized() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &true).unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        execute::unpause(deps.as_mut(), info).expect_err("Unauthorized");

        let contract = deps.api.addr_make("anyone").to_string();
        let sender = deps.api.addr_make("sender").to_string();
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert_eq!(true, res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            assert_eq!(false, res.can_execute());
        }
    }

    #[test]
    fn pause_pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let contract = deps.api.addr_make("anyone").to_string();
        let method = "any_method".to_string();

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        execute::pause(deps.as_mut(), info.clone()).unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert_eq!(true, res.is_paused());

        execute::pause(deps.as_mut(), info).unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert_eq!(true, res.is_paused());

        let sender = deps.api.addr_make("sender").to_string();
        let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        assert_eq!(false, res.can_execute());
    }
}
