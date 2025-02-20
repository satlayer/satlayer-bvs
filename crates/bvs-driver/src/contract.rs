use crate::{
    error::ContractError,
    msg::ExecuteMsg,
    msg::InstantiateMsg,
    msg::QueryMsg,
    state::{IS_BVS_CONTRACT_REGISTERED, OWNER},
};

use cosmwasm_std::{
    entry_point, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
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

    let response = Response::new().add_attribute("method", "instantiate");

    Ok(response)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ExecuteBvsOffchain { task_id } => execute_bvs_offchain(deps, info, task_id),
        ExecuteMsg::AddRegisteredBvsContract { address } => {
            add_registered_bvs_contract(deps, info, Addr::unchecked(address))
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            transfer_ownership(deps, info, new_owner_addr)
        }
    }
}

pub fn execute_bvs_offchain(
    deps: DepsMut,
    info: MessageInfo,
    task_id: String,
) -> Result<Response, ContractError> {
    let sender = info.sender;
    let is_registered = IS_BVS_CONTRACT_REGISTERED
        .may_load(deps.storage, &sender)?
        .unwrap_or(false);

    if !is_registered {
        return Err(ContractError::BvsContractNotRegistered {});
    }

    Ok(Response::new().add_event(
        Event::new("ExecuteBVSOffchain")
            .add_attribute("sender", sender.to_string())
            .add_attribute("task_id", task_id.to_string()),
    ))
}

pub fn add_registered_bvs_contract(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    IS_BVS_CONTRACT_REGISTERED.save(deps.storage, &Addr::unchecked(address.clone()), &true)?;

    Ok(Response::new().add_event(
        Event::new("add_registered_bvs_contract")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("address", address.to_string()),
    ))
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    let current_owner = OWNER.load(deps.storage)?;

    if current_owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    OWNER.save(deps.storage, &new_owner)?;

    Ok(Response::new()
        .add_attribute("method", "transfer_ownership")
        .add_attribute("new_owner", new_owner.to_string()))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
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
        let msg = InstantiateMsg {
            initial_owner: owner,
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.attributes.len());
        assert_eq!(("method", "instantiate"), res.attributes[0]);
    }

    #[test]
    fn test_executebvsoffchain() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let bvs_contract = Addr::unchecked("bvs_contract");
        let info = message_info(&bvs_contract, &[]);
        let task_id = "1000".to_string();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddRegisteredBvsContract {
                address: bvs_contract.to_string(),
            },
        )
        .unwrap();

        let msg = ExecuteMsg::ExecuteBvsOffchain {
            task_id: task_id.clone(),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(1, res.events.len());

        assert_eq!("ExecuteBVSOffchain", res.events[0].ty);

        assert_eq!(
            res.events[0].attributes,
            vec![
                cosmwasm_std::Attribute {
                    key: "sender".to_string(),
                    value: bvs_contract.to_string(),
                },
                cosmwasm_std::Attribute {
                    key: "task_id".to_string(),
                    value: task_id.clone(),
                },
            ]
        );
    }

    #[test]
    fn test_create_executebvsoffchain_msg() {
        let contract_addr = "contract123".to_string();
        let task_id = 1000.to_string();

        let msg = ExecuteMsg::ExecuteBvsOffchain {
            task_id: task_id.clone(),
        };

        let cosmos_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_json_binary(&msg).unwrap(),
            funds: vec![],
        });

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
                    ExecuteMsg::ExecuteBvsOffchain { task_id: id } => {
                        assert_eq!(id, task_id);
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
        let info = message_info(&Addr::unchecked("admin"), &[]);
        let bvs_contract_address = "bvs_contract_123";
        let msg = ExecuteMsg::AddRegisteredBvsContract {
            address: bvs_contract_address.to_string(),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(1, res.events.len());
        assert_eq!("add_registered_bvs_contract", res.events[0].ty);
        assert_eq!(
            vec![("sender", "admin"), ("address", bvs_contract_address),],
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
    fn test_transfer_ownership() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let initial_owner = deps.api.addr_make("initial_owner");
        let init_msg = InstantiateMsg {
            initial_owner: initial_owner.to_string(),
        };
        let init_info = message_info(&Addr::unchecked("creator"), &[]);
        instantiate(deps.as_mut(), env.clone(), init_info, init_msg).unwrap();

        let info = message_info(&initial_owner, &[]);
        let new_owner = deps.api.addr_make("new_owner").to_string();
        let msg = ExecuteMsg::TransferOwnership {
            new_owner: new_owner.clone(),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(2, res.attributes.len());
        assert_eq!(
            vec![
                ("method", "transfer_ownership"),
                ("new_owner", new_owner.as_str())
            ],
            res.attributes
        );
    }
}
