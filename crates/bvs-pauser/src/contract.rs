#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::PauserError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::PAUSED;
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PauserError> {
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
) -> Result<Response, PauserError> {
    match msg {
        ExecuteMsg::Pause {} => execute::pause(deps, info),
        ExecuteMsg::Unpause {} => execute::unpause(deps, info),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(PauserError::Ownership)
        }
    }
}

mod execute {
    use super::*;
    use crate::state::PAUSED;

    pub fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, PauserError> {
        ownership::assert_owner(deps.storage, &info)?;

        PAUSED.save(deps.storage, &true)?;
        Ok(Response::new()
            .add_attribute("method", "pause")
            .add_attribute("sender", info.sender))
    }

    pub fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, PauserError> {
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
            let contract = deps.api.addr_validate(&contract)?;
            to_json_binary(&query::is_paused(deps, contract, method)?)
        }
        QueryMsg::CanExecute {
            contract,
            sender,
            method,
        } => {
            let contract = deps.api.addr_validate(&contract)?;
            let sender = deps.api.addr_validate(&sender)?;
            to_json_binary(&query::can_execute(deps, contract, sender, method)?)
        }
    }
}

mod query {
    use super::*;
    use crate::msg::{CanExecuteFlag, CanExecuteResponse, IsPausedResponse};
    use crate::state::PAUSED;
    use cosmwasm_std::Addr;

    /// TODO(future): The `_contract` and `_method` are currently not used.
    ///  To implement checking of paused status against contract and method.
    ///  Added for future compatibility, not yet utilized—current design pauses all execute.
    pub fn is_paused(deps: Deps, _contract: Addr, _method: String) -> StdResult<IsPausedResponse> {
        let is_paused = PAUSED.load(deps.storage)?;
        Ok(IsPausedResponse::new(is_paused))
    }

    /// TODO(future): The `_contract`, `_sender` and `_method` are currently not used.
    ///  To implement checking of paused status against contract, method and sender.
    ///  Added for future compatibility, not yet utilized—current design pauses all execute.
    pub fn can_execute(
        deps: Deps,
        _contract: Addr,
        _sender: Addr,
        _method: String,
    ) -> StdResult<CanExecuteResponse> {
        let is_paused = PAUSED.load(deps.storage)?;
        if is_paused {
            return Ok(CanExecuteFlag::Paused.into());
        }
        Ok(CanExecuteFlag::CanExecute.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{CanExecuteFlag, IsPausedResponse};
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
        assert!(!value.is_paused());
    }

    #[test]
    fn pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        let info = message_info(&owner, &[]);
        execute::pause(deps.as_mut(), info).unwrap();

        let contract = deps.api.addr_make("contract");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        let response = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(response.is_paused());

        let response = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        let flag: CanExecuteFlag = response.into();
        assert_eq!(flag, CanExecuteFlag::Paused);
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

        let contract = deps.api.addr_make("contract");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert!(!res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            let flag: CanExecuteFlag = res.into();
            assert_eq!(flag, CanExecuteFlag::CanExecute);
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

        let contract = deps.api.addr_make("anyone");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert!(!res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            let flag: CanExecuteFlag = res.into();
            assert_eq!(flag, CanExecuteFlag::CanExecute);
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

        let contract = deps.api.addr_make("anyone");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        {
            let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
            assert!(res.is_paused());

            let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
            let flag: CanExecuteFlag = res.into();
            assert_eq!(flag, CanExecuteFlag::Paused);
        }
    }

    #[test]
    fn pause_pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let contract = deps.api.addr_make("anyone");
        let method = "any_method".to_string();

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        execute::pause(deps.as_mut(), info.clone()).unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(res.is_paused());

        execute::pause(deps.as_mut(), info).unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(res.is_paused());

        let sender = deps.api.addr_make("sender");
        let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::Paused);
    }
}
