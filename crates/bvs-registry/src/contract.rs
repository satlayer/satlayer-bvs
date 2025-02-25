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
        QueryMsg::IsPaused { contract, method } => {
            to_json_binary(&query::is_paused(deps, contract, method)?)
        }
    }
}

pub mod query {
    use super::*;
    use crate::msg::IsPausedResponse;
    use crate::state::PAUSED;

    /// The sender (contract) and method are currently not used.
    /// TODO(future): implement checking of paused status against the sender and method
    pub fn is_paused(
        deps: Deps,
        _contract: String,
        _method: String,
    ) -> StdResult<IsPausedResponse> {
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
                contract: deps.api.addr_make("anyone").to_string(),
                method: "any_method".to_string(),
            },
        )
        .unwrap();
        let value: IsPausedResponse = from_json(&res).unwrap();
        assert_eq!(false, value.paused);
    }

    #[test]
    fn pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        OWNER.save(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        let info = message_info(&owner, &[]);
        execute::pause(deps.as_mut(), info).unwrap();

        let contract = deps.api.addr_make("anyone").to_string();
        let method = "any_method".to_string();

        let response = query::is_paused(deps.as_ref(), contract, method).unwrap();
        assert_eq!(true, response.paused);
    }

    #[test]
    fn pause_unauthorized() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        OWNER.save(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        execute::pause(deps.as_mut(), info).expect_err("Unauthorized");

        let contract = deps.api.addr_make("contract").to_string();
        let method = "any_method".to_string();
        let response = query::is_paused(deps.as_ref(), contract, method).unwrap();
        assert_eq!(false, response.paused);
    }

    #[test]
    fn unpause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        OWNER.save(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &true).unwrap();

        let info = message_info(&owner, &[]);
        execute::unpause(deps.as_mut(), info).unwrap();

        let contract = deps.api.addr_make("anyone").to_string();
        let method = "any_method".to_string();

        let response = query::is_paused(deps.as_ref(), contract, method).unwrap();
        assert_eq!(false, response.paused);
    }

    #[test]
    fn unpause_unauthorized() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");

        OWNER.save(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &true).unwrap();

        let not_owner = deps.api.addr_make("not_owner");
        let info = message_info(&not_owner, &[]);
        execute::unpause(deps.as_mut(), info).expect_err("Unauthorized");

        let contract = deps.api.addr_make("contract").to_string();
        let method = "any_method".to_string();
        let response = query::is_paused(deps.as_ref(), contract, method).unwrap();
        assert_eq!(true, response.paused);
    }

    #[test]
    fn pause_pause() {
        let mut deps = mock_dependencies();

        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);
        let contract = deps.api.addr_make("anyone").to_string();
        let method = "any_method".to_string();

        OWNER.save(&mut deps.storage, &owner).unwrap();
        PAUSED.save(&mut deps.storage, &false).unwrap();

        execute::pause(deps.as_mut(), info.clone()).unwrap();

        let response = query::is_paused(deps.as_ref(), contract.clone(), method.clone()).unwrap();
        assert_eq!(true, response.paused);

        execute::pause(deps.as_mut(), info).unwrap();

        let response = query::is_paused(deps.as_ref(), contract, method).unwrap();
        assert_eq!(true, response.paused);
    }
}
