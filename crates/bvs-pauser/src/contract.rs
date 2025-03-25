#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::PauserError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PauserError> {
    match msg {
        ExecuteMsg::Pause { method, contract } => {
            let contract = deps.api.addr_validate(&contract)?;
            execute::pause(deps, env, info, contract, method)
        }
        ExecuteMsg::Unpause { method, contract } => {
            let contract = deps.api.addr_validate(&contract)?;
            execute::unpause(deps, info, contract, method)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(PauserError::Ownership)
        }
    }
}

mod execute {
    use cosmwasm_std::Addr;

    use super::*;
    use crate::state::PAUSED;

    pub fn pause(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: Addr,
        method: String,
    ) -> Result<Response, PauserError> {
        ownership::assert_owner(deps.storage, &info)?;

        // Check if the contract is already paused
        // Only mutate the state if it is not already paused
        match PAUSED.load(deps.storage, (&contract, &method)) {
            Ok(_) => {}
            Err(_) => {
                PAUSED.save(deps.storage, (&contract, &method), &env.block.height)?;
            }
        };

        Ok(Response::new()
            .add_attribute("method", "pause")
            .add_attribute("sender", info.sender))
    }

    pub fn unpause(
        deps: DepsMut,
        info: MessageInfo,
        contract: Addr,
        method: String,
    ) -> Result<Response, PauserError> {
        ownership::assert_owner(deps.storage, &info)?;

        // Check if the contract is already paused
        // Only mutate the state if it is not already paused
        match PAUSED.load(deps.storage, (&contract, &method)) {
            Ok(_) => {
                PAUSED.remove(deps.storage, (&contract, &method));
            }
            Err(_) => {}
        }

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
    pub fn is_paused(deps: Deps, contract: Addr, method: String) -> StdResult<IsPausedResponse> {
        let is_paused = match PAUSED.load(deps.storage, (&contract, &method)) {
            Ok(_) => true,
            Err(_) => false,
        };
        Ok(IsPausedResponse::new(is_paused))
    }

    /// TODO(future): The `_contract`, `_sender` and `_method` are currently not used.
    ///  To implement checking of paused status against contract, method and sender.
    ///  Added for future compatibility, not yet utilized—current design pauses all execute.
    pub fn can_execute(
        deps: Deps,
        contract: Addr,
        _sender: Addr,
        method: String,
    ) -> StdResult<CanExecuteResponse> {
        let is_paused = match PAUSED.load(deps.storage, (&contract, &method)) {
            Ok(_) => true,
            Err(_) => false,
        };

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
    use crate::state::PAUSED;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn instantiate_msg() {
        let mut deps = mock_dependencies();

        let info = message_info(&deps.api.addr_make("sender"), &coins(1000, "rat"));
        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
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
        let env = mock_env();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();

        let info = message_info(&owner, &[]);
        let contract = deps.api.addr_make("contract");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        execute::pause(deps.as_mut(), env, info, contract.clone(), method.clone()).unwrap();

        let response = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(response.is_paused());

        let response = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        let flag: CanExecuteFlag = response.into();
        assert_eq!(flag, CanExecuteFlag::Paused);
    }

    #[test]
    fn pause_unauthorized() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");

        ownership::set_owner(&mut deps.storage, &owner).unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        let contract = deps.api.addr_make("contract");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();
        execute::pause(deps.as_mut(), env, info, contract.clone(), method.clone())
            .expect_err("Unauthorized");

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
        let env = mock_env();

        let owner = deps.api.addr_make("owner");

        let contract = deps.api.addr_make("anyone");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED
            .save(&mut deps.storage, (&contract, &method), &env.block.height)
            .unwrap();

        let info = message_info(&owner, &[]);
        execute::unpause(deps.as_mut(), info, contract.clone(), method.clone()).unwrap();

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
        let env = mock_env();

        let owner = deps.api.addr_make("owner");

        let contract = deps.api.addr_make("anyone");
        let sender = deps.api.addr_make("sender");
        let method = "any_method".to_string();

        ownership::set_owner(&mut deps.storage, &owner).unwrap();
        PAUSED
            .save(&mut deps.storage, (&contract, &method), &env.block.height)
            .unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        execute::unpause(deps.as_mut(), info, contract.clone(), method.clone())
            .expect_err("Unauthorized");

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
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let contract = deps.api.addr_make("anyone");
        let method = "any_method".to_string();

        ownership::set_owner(&mut deps.storage, &owner).unwrap();

        execute::pause(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            contract.clone(),
            method.clone(),
        )
        .unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(res.is_paused());

        execute::pause(deps.as_mut(), env, info, contract.clone(), method.clone()).unwrap();

        let res = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert!(res.is_paused());

        let sender = deps.api.addr_make("sender");
        let res = query::can_execute(deps.as_ref(), contract, sender, method).unwrap();
        let flag: CanExecuteFlag = res.into();
        assert_eq!(flag, CanExecuteFlag::Paused);
    }
}
