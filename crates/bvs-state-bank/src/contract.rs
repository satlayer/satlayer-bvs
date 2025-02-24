use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::ValueResponse,
    state::{BVS_DIRECTORY, IS_BVS_CONTRACT_REGISTERED, OWNER, PENDING_OWNER, VALUES},
};

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "BVS State Bank";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.initial_owner)?;
    OWNER.save(deps.storage, &owner)?;

    let bvs_directory = deps.api.addr_validate(&msg.bvs_directory)?;
    BVS_DIRECTORY.save(deps.storage, &bvs_directory)?;

    let response = Response::new().add_attribute("method", "instantiate");

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Set { key, value } => execute_set(deps, info, key, value),
        ExecuteMsg::AddRegisteredBvsContract { address } => {
            add_registered_bvs_contract(deps, info, Addr::unchecked(address))
        }
        ExecuteMsg::TwoStepTransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            two_step_transfer_ownership(deps, info, new_owner_addr)
        }
        ExecuteMsg::AcceptOwnership {} => accept_ownership(deps, info),
        ExecuteMsg::CancelOwnershipTransfer {} => cancel_ownership_transfer(deps, info),
    }
}

pub fn execute_set(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let is_registered = IS_BVS_CONTRACT_REGISTERED
        .may_load(deps.storage, &sender)?
        .unwrap_or(false);

    if !is_registered {
        return Err(ContractError::BvsContractNotRegistered {});
    }

    VALUES.save(deps.storage, key.clone(), &value)?;

    Ok(Response::new().add_event(
        Event::new("UpdateState")
            .add_attribute("sender", sender.to_string())
            .add_attribute("key", key)
            .add_attribute("value", value.to_string()),
    ))
}

pub fn add_registered_bvs_contract(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    only_directory(deps.as_ref(), &info)?;

    IS_BVS_CONTRACT_REGISTERED.save(deps.storage, &Addr::unchecked(address.clone()), &true)?;

    Ok(Response::new().add_event(
        Event::new("add_registered_bvs_contract")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("address", address.to_string()),
    ))
}

pub fn set_bvs_directory(
    deps: DepsMut,
    info: MessageInfo,
    new_directory: String,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let new_directory_addr = deps.api.addr_validate(&new_directory)?;

    BVS_DIRECTORY.save(deps.storage, &new_directory_addr)?;

    Ok(Response::new()
        .add_attribute("action", "set_bvs_directory")
        .add_attribute("new_directory", new_directory))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Get { key } => query_value(deps, key),
    }
}

fn query_value(deps: Deps, key: String) -> StdResult<Binary> {
    let result = VALUES.may_load(deps.storage, key)?;

    if let Some(value) = result {
        return to_json_binary(&ValueResponse { value });
    }

    Err(StdError::generic_err("Value not found"))
}

pub fn two_step_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    PENDING_OWNER.save(deps.storage, &Some(new_owner.clone()))?;

    let resp = Response::new()
        .add_attribute("action", "two_step_transfer_ownership")
        .add_attribute("old_owner", info.sender.to_string())
        .add_attribute("pending_owner", new_owner.to_string());

    Ok(resp)
}

pub fn accept_ownership(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let pending_owner = PENDING_OWNER.load(deps.storage)?;

    let pending_owner_addr = match pending_owner {
        Some(addr) => addr,
        None => return Err(ContractError::NoPendingOwner {}),
    };

    if info.sender != pending_owner_addr {
        return Err(ContractError::Unauthorized {});
    }

    OWNER.save(deps.storage, &info.sender)?;
    PENDING_OWNER.save(deps.storage, &None)?;

    let resp = Response::new()
        .add_attribute("action", "accept_ownership")
        .add_attribute("new_owner", info.sender.to_string());

    Ok(resp)
}

pub fn cancel_ownership_transfer(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    PENDING_OWNER.save(deps.storage, &None)?;

    let resp = Response::new().add_attribute("action", "cancel_ownership_transfer");

    Ok(resp)
}

pub fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn only_directory(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let directory = BVS_DIRECTORY.load(deps.storage)?;
    if info.sender != directory {
        return Err(ContractError::NotBVSDirectory {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies, mock_env},
        to_json_binary, Addr, CosmosMsg, WasmMsg,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let owner = deps.api.addr_make("owner").to_string();
        let bvs_directory = deps.api.addr_make("bvs_directory").to_string();

        let msg = InstantiateMsg {
            initial_owner: owner,
            bvs_directory,
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.attributes.len());
        assert_eq!(("method", "instantiate"), res.attributes[0]);
    }

    #[test]
    fn test_set_and_get() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let directory = deps.api.addr_make("directory");
        let init_msg = InstantiateMsg {
            initial_owner: owner.to_string(),
            bvs_directory: directory.to_string(),
        };
        let init_info = message_info(&Addr::unchecked("creator"), &[]);
        instantiate(deps.as_mut(), env.clone(), init_info, init_msg).unwrap();

        let directory_info = message_info(&directory, &[]);
        let register_msg = ExecuteMsg::AddRegisteredBvsContract {
            address: "alice".to_string(),
        };
        execute(deps.as_mut(), env.clone(), directory_info, register_msg).unwrap();

        let info = message_info(&Addr::unchecked("alice"), &[]);
        let msg = ExecuteMsg::Set {
            key: "temperature".to_string(),
            value: 25.to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(1, res.events.len());
        assert_eq!("UpdateState", res.events[0].ty);
        assert_eq!(
            vec![("sender", "alice"), ("key", "temperature"), ("value", "25"),],
            res.events[0].attributes
        );

        let query_msg = QueryMsg::Get {
            key: "temperature".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let res: ValueResponse = from_json(res).unwrap();
        assert_eq!("25", res.value);

        let query_msg = QueryMsg::Get {
            key: "non_existent".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg);
        assert!(res.is_err());

        let unregistered_info = message_info(&Addr::unchecked("bob"), &[]);
        let unregistered_msg = ExecuteMsg::Set {
            key: "pressure".to_string(),
            value: 1013.to_string(),
        };
        let unregistered_res = execute(deps.as_mut(), env, unregistered_info, unregistered_msg);
        assert!(unregistered_res.is_err());
    }

    fn _create_set_msg(contract_addr: String, key: String, value: String) -> StdResult<CosmosMsg> {
        let msg = ExecuteMsg::Set { key, value };
        let wasm_msg = WasmMsg::Execute {
            contract_addr,
            msg: to_json_binary(&msg)?,
            funds: vec![],
        };
        Ok(CosmosMsg::Wasm(wasm_msg))
    }

    #[test]
    fn test_create_set_msg() {
        let contract_addr = "contract123".to_string();
        let key = "pressure".to_string();
        let value = 1013.to_string();

        let cosmos_msg =
            _create_set_msg(contract_addr.clone(), key.clone(), value.clone()).unwrap();

        match cosmos_msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr,
                msg,
                funds,
            }) => {
                assert_eq!(addr, contract_addr);
                assert_eq!(funds, vec![]);

                let parsed_msg: ExecuteMsg = from_json(msg).unwrap();
                match parsed_msg {
                    ExecuteMsg::Set { key: k, value: v } => {
                        assert_eq!(k, key);
                        assert_eq!(v, value);
                    }
                    _ => panic!("Unexpected ExecuteMsg type"),
                }
            }
            _ => panic!("Unexpected CosmosMsg type"),
        }
    }

    #[test]
    fn test_add_registered_bvs_contract() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let directory = deps.api.addr_make("directory");

        let init_msg = InstantiateMsg {
            initial_owner: owner.to_string(),
            bvs_directory: directory.to_string(),
        };
        let init_info = message_info(&Addr::unchecked("creator"), &[]);
        instantiate(deps.as_mut(), env.clone(), init_info, init_msg).unwrap();

        let directory_addr = Addr::unchecked(&directory);
        let info = message_info(&directory_addr, &[]);
        let bvs_contract_address = "bvs_contract_123".to_string();
        let msg = ExecuteMsg::AddRegisteredBvsContract {
            address: bvs_contract_address.clone(),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(1, res.events.len());
        assert_eq!("add_registered_bvs_contract", res.events[0].ty);
        assert_eq!(
            vec![
                ("sender", directory_addr.to_string()),
                ("address", bvs_contract_address.clone()),
            ],
            res.events[0].attributes
        );

        let is_registered = IS_BVS_CONTRACT_REGISTERED
            .may_load(
                deps.as_ref().storage,
                &Addr::unchecked(bvs_contract_address),
            )
            .unwrap()
            .unwrap_or(false);
        assert!(is_registered);
    }

    #[test]
    fn test_two_step_transfer_ownership() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("initial_owner");
        let bvs_directory = deps.api.addr_make("bvs_directory");

        let init_msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
            bvs_directory: bvs_directory.to_string(),
        };

        let init_info = message_info(&Addr::unchecked("creator"), &[]);
        instantiate(deps.as_mut(), env.clone(), init_info, init_msg).unwrap();

        let old_owner_info = message_info(&initial_owner, &[]);

        let new_owner_addr = deps.api.addr_make("new_owner");
        let msg = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: new_owner_addr.to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), old_owner_info.clone(), msg).unwrap();

        assert_eq!(res.attributes.len(), 3);
        assert_eq!(res.attributes[0], ("action", "two_step_transfer_ownership"));
        assert_eq!(res.attributes[1], ("old_owner", initial_owner.to_string()));
        assert_eq!(
            res.attributes[2],
            ("pending_owner", new_owner_addr.to_string())
        );

        let cancel_msg = ExecuteMsg::CancelOwnershipTransfer {};
        let cancel_res = execute(
            deps.as_mut(),
            env.clone(),
            old_owner_info.clone(),
            cancel_msg,
        )
        .unwrap();

        assert_eq!(cancel_res.attributes.len(), 1);
        assert_eq!(
            cancel_res.attributes[0],
            ("action", "cancel_ownership_transfer")
        );

        let msg2 = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: new_owner_addr.to_string(),
        };
        execute(deps.as_mut(), env.clone(), old_owner_info.clone(), msg2).unwrap();

        let new_owner_info = message_info(&new_owner_addr, &[]);

        let accept_msg = ExecuteMsg::AcceptOwnership {};
        let accept_res = execute(
            deps.as_mut(),
            env.clone(),
            new_owner_info.clone(),
            accept_msg,
        )
        .unwrap();

        assert_eq!(accept_res.attributes.len(), 2);
        assert_eq!(accept_res.attributes[0], ("action", "accept_ownership"));
        assert_eq!(
            accept_res.attributes[1],
            ("new_owner", new_owner_addr.to_string())
        );

        let stored_owner = OWNER.load(&deps.storage).unwrap();
        assert_eq!(stored_owner, new_owner_addr);

        let pending_owner = PENDING_OWNER.load(&deps.storage).unwrap();
        assert_eq!(pending_owner, None);

        let someone_else = deps.api.addr_make("someone_else").to_string();
        let msg3 = ExecuteMsg::TwoStepTransferOwnership {
            new_owner: someone_else,
        };
        let err = execute(deps.as_mut(), env.clone(), old_owner_info.clone(), msg3).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("Expected Unauthorized error, got: {:?}", e),
        }
    }
}
