#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{OWNER, PAUSED};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "BVS Registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    OWNER.save(deps.storage, &owner)?;
    PAUSED.save(deps.storage, &msg.initial_paused)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
    }
}

pub mod execute {
    use super::*;
    use crate::state::{OWNER, PAUSED};

    pub fn pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        only_owner(deps.as_ref(), &info)?;

        PAUSED.save(deps.storage, &true)?;
        Ok(Response::new()
            .add_attribute("method", "pause")
            .add_attribute("sender", info.sender))
    }

    pub fn unpause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        only_owner(deps.as_ref(), &info)?;

        PAUSED.save(deps.storage, &false)?;
        Ok(Response::new()
            .add_attribute("method", "unpause")
            .add_attribute("sender", info.sender))
    }

    fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {});
        }
        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsPaused { sender, method } => {
            to_json_binary(&query::is_paused(deps, sender, method)?)
        }
    }
}

pub mod query {
    use super::*;
    use crate::msg::IsPausedResponse;
    use crate::state::PAUSED;

    /// The sender and method are currently not used.
    /// Added for future checking of paused status against the sender and method.
    pub fn is_paused(deps: Deps, _sender: String, _method: String) -> StdResult<IsPausedResponse> {
        let state = PAUSED.load(deps.storage)?;
        Ok(IsPausedResponse { paused: state })
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
                sender: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let value: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(false, value.paused);
    }

    #[test]
    fn execute_pause() {
        let mut deps = mock_dependencies();

        let info = message_info(&deps.api.addr_make("owner"), &coins(1000, "rat"));
        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
            initial_paused: false,
        };
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Pause {},
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsPaused {
                sender: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let response: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(true, response.paused);
    }

    #[test]
    fn execute_pause_unauthorized() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
            initial_paused: false,
        };
        let info = message_info(&deps.api.addr_make("owner"), &coins(1000, "rat"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&deps.api.addr_make("not_owner"), &coins(1000, "rat"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Pause {}).expect_err("Unauthorized");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsPaused {
                sender: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let response: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(false, response.paused);
    }

    #[test]
    fn execute_unpause() {
        let mut deps = mock_dependencies();

        let info = message_info(&deps.api.addr_make("owner"), &coins(1000, "rat"));
        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
            initial_paused: true,
        };
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Unpause {},
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsPaused {
                sender: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let response: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(false, response.paused);
    }

    #[test]
    fn execute_unpause_unauthorized() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: deps.api.addr_make("owner").to_string(),
            initial_paused: true,
        };
        let info = message_info(&deps.api.addr_make("owner"), &coins(1000, "rat"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&deps.api.addr_make("not_owner"), &coins(1000, "rat"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Unpause {}).expect_err("Unauthorized");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsPaused {
                sender: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let response: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(true, response.paused);
    }
}
