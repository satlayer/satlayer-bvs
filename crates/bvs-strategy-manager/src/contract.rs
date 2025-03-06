#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    auth,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state,
    state::{STAKER_STRATEGY_LIST, STAKER_STRATEGY_SHARES},
};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};

use crate::msg::delegation_manager::{self, IncreaseDelegatedShares};
use bvs_library::ownership;
use bvs_strategy_base::{
    msg::ExecuteMsg as BaseExecuteMsg,
    msg::QueryMsg as BaseQueryMsg,
    msg::{TotalSharesResponse, UnderlyingTokenResponse},
};

const CONTRACT_NAME: &str = "BVS Strategy Manager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum length of the strategy list for a staker
/// This value can be changed in the future
pub const MAX_STRATEGY_LENGTH: usize = 10;

/// Submsg id for the deposit into strategy reply
pub const DEPOSIT_SUBMSG_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let registry = deps.api.addr_validate(&msg.registry)?;
    bvs_registry::api::set_registry_addr(deps.storage, &registry)?;

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
) -> Result<Response, ContractError> {
    bvs_registry::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::DepositIntoStrategy {
            strategy,
            token,
            amount,
        } => {
            let strategy_addr = deps.api.addr_validate(&strategy)?;
            let token_addr = deps.api.addr_validate(&token)?;

            let staker = info.sender.clone();
            deposit_into_strategy(deps, info, staker, strategy_addr, token_addr, amount)
        }
        ExecuteMsg::WithdrawSharesAsTokens {
            recipient,
            strategy,
            shares,
        } => {
            let recipient = deps.api.addr_validate(&recipient)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            withdraw_shares_as_tokens(deps, info, recipient, strategy, shares)
        }
        ExecuteMsg::RemoveShares {
            staker,
            strategy,
            shares,
        } => {
            let staker = deps.api.addr_validate(&staker)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            remove_shares(deps, info, staker, strategy, shares)
        }
        ExecuteMsg::AddShares {
            staker,
            strategy,
            shares,
        } => {
            let staker = deps.api.addr_validate(&staker)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            add_shares(deps, info, staker, strategy, shares)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
        ExecuteMsg::SetRouting {
            delegation_manager,
            slash_manager,
        } => {
            let delegation_manager = deps.api.addr_validate(&delegation_manager)?;
            let slash_manager = deps.api.addr_validate(&slash_manager)?;

            auth::set_routing(deps, info, delegation_manager, slash_manager)
        }
        ExecuteMsg::AddStrategy {
            strategy,
            whitelisted,
        } => {
            let strategy = deps.api.addr_validate(&strategy)?;
            execute::add_strategy(deps, env, info, strategy, whitelisted)
        }
        ExecuteMsg::UpdateStrategy {
            strategy,
            whitelisted,
        } => {
            let strategy = deps.api.addr_validate(&strategy)?;
            execute::update_strategy(deps, info, strategy, whitelisted)
        }
    }
}

mod execute {
    use crate::state::STRATEGY_WHITELISTED;
    use crate::ContractError;
    use bvs_library::ownership;
    use bvs_strategy_base::{msg::QueryMsg as BaseQueryMsg, msg::StrategyManagerResponse};
    use cosmwasm_std::{
        to_json_binary, Addr, DepsMut, Env, Event, MessageInfo, Response, WasmQuery,
    };

    /// Add a new strategy, setting whitelisted=true will allow staker to deposit into the strategy.
    /// Only the owner can add a new strategy.
    pub fn add_strategy(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        strategy: Addr,
        whitelisted: bool,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        // Check if the contract is properly initiated on the chain
        let StrategyManagerResponse(strategy_manager) = deps.querier.query(
            &WasmQuery::Smart {
                contract_addr: strategy.to_string(),
                msg: to_json_binary(&BaseQueryMsg::StrategyManager {})?,
            }
            .into(),
        )?;

        if strategy_manager != env.contract.address {
            return Err(ContractError::InvalidStrategy {
                msg: "Strategy manager mismatch".to_string(),
            });
        }

        STRATEGY_WHITELISTED.save(deps.storage, &strategy, &whitelisted)?;

        Ok(Response::new().add_event(
            Event::new("StrategyUpdated")
                .add_attribute("strategy", strategy.to_string())
                .add_attribute("whitelisted", whitelisted.to_string()),
        ))
    }

    /// Update an existing strategy, setting whitelisted=true will allow staker to deposit into the strategy.
    /// Only the owner can update a strategy.
    pub fn update_strategy(
        deps: DepsMut,
        info: MessageInfo,
        strategy: Addr,
        whitelisted: bool,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        STRATEGY_WHITELISTED.save(deps.storage, &strategy, &whitelisted)?;

        Ok(Response::new().add_event(
            Event::new("StrategyUpdated")
                .add_attribute("strategy", strategy.to_string())
                .add_attribute("whitelisted", whitelisted.to_string()),
        ))
    }
}

pub fn deposit_into_strategy(
    mut deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    token: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    state::assert_strategy_whitelisted(deps.as_ref(), &strategy)?;

    let payload = to_json_binary(&(&staker, &strategy))?;
    let submsg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: strategy.to_string(),
            msg: to_json_binary(&BaseExecuteMsg::Deposit {
                sender: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        },
        DEPOSIT_SUBMSG_ID,
    )
    .with_payload(payload);

    Ok(Response::new().add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, msg: cosmwasm_std::Reply) -> StdResult<Response> {
    if msg.id != DEPOSIT_SUBMSG_ID {
        return Err(StdError::generic_err("Invalid submsg id"));
    }
    let (staker, strategy): (Addr, Addr) = from_json(msg.payload)?;
    let events = msg.result.unwrap().events;
    let new_shares = events
        .iter()
        .find_map(|event| {
            if event.ty == "Deposit" {
                event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "new_shares")
                    .map(|attr| attr.value.parse::<Uint128>().unwrap())
            } else {
                None
            }
        })
        .unwrap();

    add_shares_internal(deps.branch(), staker.clone(), strategy.clone(), new_shares).unwrap();
    let delegation_manager = auth::get_delegation_manager(deps.storage).unwrap();
    let increase_delegated_shares_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: delegation_manager.to_string(),
        msg: to_json_binary(&delegation_manager::ExecuteMsg::IncreaseDelegatedShares(
            IncreaseDelegatedShares {
                staker: staker.to_string(),
                strategy: strategy.to_string(),
                shares: new_shares,
            },
        ))?,
        funds: vec![],
    });

    Ok(Response::new().add_message(increase_delegated_shares_msg))
}

pub fn add_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;

    add_shares_internal(deps, staker, strategy, shares)
}

fn add_shares_internal(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    if shares.is_zero() {
        return Err(ContractError::InvalidShares {});
    }

    let mut strategy_list = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    let current_shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or_else(Uint128::zero);

    if current_shares.is_zero() {
        if strategy_list.len() >= MAX_STRATEGY_LENGTH {
            return Err(ContractError::MaxStrategyListLengthExceeded {});
        }
        strategy_list.push(strategy.clone());
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
    }

    let new_shares = current_shares + shares;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &new_shares)?;

    let event = Event::new("add_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string());

    Ok(Response::new().add_event(event))
}

pub fn remove_shares(
    deps: DepsMut,
    info: MessageInfo,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;
    let strategy_removed = remove_shares_internal(deps, staker.clone(), strategy.clone(), shares)?;

    let response = Response::new()
        .add_attribute("method", "remove_shares")
        .add_attribute("staker", staker.to_string())
        .add_attribute("strategy", strategy.to_string())
        .add_attribute("shares", shares.to_string())
        .add_attribute("strategy_removed", strategy_removed.to_string());

    Ok(response)
}

fn remove_shares_internal(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<bool, ContractError> {
    if shares.is_zero() {
        return Err(ContractError::InvalidShares {});
    }

    let mut current_shares = STAKER_STRATEGY_SHARES
        .may_load(deps.storage, (&staker, &strategy))?
        .unwrap_or_else(Uint128::zero);

    if shares > current_shares {
        return Err(ContractError::InvalidShares {});
    }

    // Subtract the shares
    current_shares = current_shares
        .checked_sub(shares)
        .map_err(|_| ContractError::InvalidShares {})?;
    STAKER_STRATEGY_SHARES.save(deps.storage, (&staker, &strategy), &current_shares)?;

    // If no existing shares, remove the strategy from the staker's list
    if current_shares.is_zero() {
        remove_strategy_from_staker_strategy_list(deps, staker.clone(), strategy.clone())?;
        return Ok(true);
    }

    Ok(false)
}

fn remove_strategy_from_staker_strategy_list(
    deps: DepsMut,
    staker: Addr,
    strategy: Addr,
) -> Result<(), ContractError> {
    let mut strategy_list = STAKER_STRATEGY_LIST
        .may_load(deps.storage, &staker)?
        .unwrap_or_else(Vec::new);

    if let Some(pos) = strategy_list.iter().position(|x| *x == strategy) {
        strategy_list.swap_remove(pos);
        STAKER_STRATEGY_LIST.save(deps.storage, &staker, &strategy_list)?;
        Ok(())
    } else {
        Err(ContractError::StrategyNotFound {})
    }
}

pub fn withdraw_shares_as_tokens(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Addr,
    strategy: Addr,
    shares: Uint128,
) -> Result<Response, ContractError> {
    auth::assert_delegation_manager(deps.as_ref(), &info)?;

    let withdraw_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy.to_string(),
        msg: to_json_binary(&BaseExecuteMsg::Withdraw {
            recipient: recipient.to_string(),
            shares,
        })?,
        funds: vec![],
    });

    let new_amount = STAKER_STRATEGY_SHARES.load(deps.storage, (&recipient, &strategy))? - shares;

    STAKER_STRATEGY_SHARES.save(deps.storage, (&recipient, &strategy), &new_amount)?;

    let response = Response::new().add_message(withdraw_msg);

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::StakerDepositList { staker } => {
            let staker = deps.api.addr_validate(&staker)?;

            to_json_binary(&query::staker_deposit_list(deps, staker)?)
        }
        QueryMsg::StakerStrategyShares { staker, strategy } => {
            let staker = deps.api.addr_validate(&staker)?;
            let strategy = deps.api.addr_validate(&strategy)?;

            to_json_binary(&query::staker_strategy_shares(deps, staker, strategy)?)
        }
        QueryMsg::StakerStrategyList { staker } => {
            let staker = deps.api.addr_validate(&staker)?;

            to_json_binary(&query::staker_strategy_list(deps, staker)?)
        }
        QueryMsg::IsStrategyWhitelisted(strategy) => {
            let strategy = deps.api.addr_validate(&strategy)?;
            to_json_binary(&query::is_strategy_whitelisted(deps, strategy)?)
        }
    }
}

mod query {
    use crate::msg::{
        IsStrategyWhitelistedResponse, StakerDepositListResponse, StakerStrategyListResponse,
        StakerStrategySharesResponse, StrategyShare,
    };
    use crate::state::{STAKER_STRATEGY_LIST, STAKER_STRATEGY_SHARES, STRATEGY_WHITELISTED};
    use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

    /// Is the strategy whitelisted for deposits?
    pub fn is_strategy_whitelisted(
        deps: Deps,
        strategy: Addr,
    ) -> StdResult<IsStrategyWhitelistedResponse> {
        let is_enabled = STRATEGY_WHITELISTED
            .may_load(deps.storage, &strategy)?
            .unwrap_or(false);
        Ok(IsStrategyWhitelistedResponse(is_enabled))
    }

    pub fn staker_strategy_shares(
        deps: Deps,
        staker: Addr,
        strategy: Addr,
    ) -> StdResult<StakerStrategySharesResponse> {
        let shares = STAKER_STRATEGY_SHARES
            .may_load(deps.storage, (&staker, &strategy))?
            .unwrap_or(Uint128::zero());
        Ok(StakerStrategySharesResponse(shares))
    }

    pub fn staker_strategy_list(deps: Deps, staker: Addr) -> StdResult<StakerStrategyListResponse> {
        let strategies = STAKER_STRATEGY_LIST
            .may_load(deps.storage, &staker)?
            .unwrap_or_else(Vec::new);
        Ok(StakerStrategyListResponse(strategies))
    }

    pub fn staker_deposit_list(deps: Deps, staker: Addr) -> StdResult<StakerDepositListResponse> {
        let strategies = STAKER_STRATEGY_LIST
            .may_load(deps.storage, &staker)?
            .unwrap_or_else(Vec::new);

        let mut list: Vec<StrategyShare> = Vec::with_capacity(strategies.len());

        for strategy in strategies {
            let shares = STAKER_STRATEGY_SHARES
                .may_load(deps.storage, (&staker, &strategy))?
                .unwrap_or_else(Uint128::zero);

            list.push(StrategyShare { strategy, shares });
        }

        Ok(StakerDepositListResponse(list))
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query, InstantiateMsg};
    use crate::msg::IsStrategyWhitelistedResponse;
    use bvs_library::ownership;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        from_json, to_json_binary, ContractResult, Event, Response, SystemError, SystemResult,
        WasmQuery,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
        };

        let info = message_info(&owner, &[]);
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("owner", owner.as_str())
        );
    }

    #[test]
    fn test_add_strategy() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = &deps.api.addr_make("owner");
        {
            ownership::set_owner(deps.as_mut().storage, owner).unwrap();
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => {
                    match from_json::<bvs_strategy_base::msg::QueryMsg>(msg).unwrap() {
                        bvs_strategy_base::msg::QueryMsg::StrategyManager { .. } => {
                            SystemResult::Ok(ContractResult::Ok(
                                to_json_binary(&bvs_strategy_base::msg::StrategyManagerResponse(
                                    env.contract.address.clone(),
                                ))
                                .unwrap(),
                            ))
                        }
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let strategy = deps.api.addr_make("strategy");

        let info = message_info(&owner, &[]);
        let env = mock_env();
        let response = execute::add_strategy(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            strategy.clone(),
            true,
        )
        .unwrap();

        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("StrategyUpdated")
                    .add_attribute("strategy", strategy.to_string())
                    .add_attribute("whitelisted", "true"),
            )
        );
    }

    #[test]
    fn test_update_strategy() {
        let mut deps = mock_dependencies();

        let owner = &deps.api.addr_make("owner");
        ownership::set_owner(deps.as_mut().storage, owner).unwrap();

        let strategy = deps.api.addr_make("strategy");

        {
            let IsStrategyWhitelistedResponse(is_whitelisted) =
                query::is_strategy_whitelisted(deps.as_ref(), strategy.clone()).unwrap();
            assert_eq!(is_whitelisted, false);
        }

        let owner_info = message_info(&owner, &[]);

        {
            let response =
                execute::update_strategy(deps.as_mut(), owner_info.clone(), strategy.clone(), true)
                    .unwrap();

            assert_eq!(
                response,
                Response::new().add_event(
                    Event::new("StrategyUpdated")
                        .add_attribute("strategy", strategy.to_string())
                        .add_attribute("whitelisted", "true"),
                )
            );

            let IsStrategyWhitelistedResponse(is_whitelisted) =
                query::is_strategy_whitelisted(deps.as_ref(), strategy.clone()).unwrap();
            assert_eq!(is_whitelisted, true);
        }

        {
            let response = execute::update_strategy(
                deps.as_mut(),
                owner_info.clone(),
                strategy.clone(),
                false,
            )
            .unwrap();

            assert_eq!(
                response,
                Response::new().add_event(
                    Event::new("StrategyUpdated")
                        .add_attribute("strategy", strategy.to_string())
                        .add_attribute("whitelisted", "false"),
                )
            );

            let IsStrategyWhitelistedResponse(is_whitelisted) =
                query::is_strategy_whitelisted(deps.as_ref(), strategy.clone()).unwrap();
            assert_eq!(is_whitelisted, false);
        }
    }
}

#[cfg(test)]
mod tests_old {
    use super::*;
    use crate::msg::{StakerDepositListResponse, StakerStrategyListResponse, StrategyShare};
    use bvs_strategy_base::{msg::QueryMsg::UnderlyingToken, msg::StrategyManagerResponse};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_json, Addr, ContractResult, OwnedDeps, SystemError, SystemResult};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "instantiate");
        assert_eq!(res.attributes[1].key, "owner");
        assert_eq!(res.attributes[1].value, owner.as_str());

        let owner = ownership::get_owner(&deps.storage).unwrap();
        assert_eq!(owner, owner.clone());
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
        };

        let delegation_manager = deps.api.addr_make("delegation_manager");
        let slasher = deps.api.addr_make("slasher");
        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();
        auth::set_routing(
            deps.as_mut(),
            owner_info.clone(),
            delegation_manager.clone(),
            slasher,
        )
        .unwrap();

        (
            deps,
            env,
            owner_info,
            message_info(&delegation_manager, &[]),
        )
    }

    #[test]
    fn test_add_new_strategy() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy");

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart {
                contract_addr: _,
                msg,
            } => {
                let query_msg: BaseQueryMsg = from_json(msg).unwrap();
                match query_msg {
                    BaseQueryMsg::StrategyManager {} => {
                        let response = StrategyManagerResponse(_env.contract.address.clone());
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = execute::add_strategy(
            deps.as_mut(),
            mock_env(),
            _owner_info.clone(),
            strategy.clone(),
            true,
        );

        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn test_deposit_into_strategy() {
        let (mut deps, _env, _, info_delegation_manager) = instantiate_contract();

        let strategy = deps.api.addr_make("strategy1");
        state::STRATEGY_WHITELISTED
            .save(&mut deps.storage, &strategy, &true)
            .unwrap();

        let token = deps.api.addr_make("token");
        let amount = Uint128::new(100);

        let strategy_for_closure = strategy.clone();
        let token_for_closure = token.clone();

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == strategy_for_closure.to_string() =>
            {
                let strategy_query_msg: BaseQueryMsg = from_json(msg).unwrap();
                match strategy_query_msg {
                    UnderlyingToken {} => {
                        let response = UnderlyingTokenResponse(token_for_closure.clone());
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&response).unwrap()))
                    }
                    BaseQueryMsg::TotalShares {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&TotalSharesResponse(Uint128::new(1000))).unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            WasmQuery::Smart { contract_addr, msg }
                if *contract_addr == token_for_closure.to_string() =>
            {
                let cw20_query_msg: Cw20QueryMsg = from_json(msg).unwrap();
                match cw20_query_msg {
                    Cw20QueryMsg::Balance { address: _ } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&Cw20BalanceResponse {
                            balance: Uint128::new(1000),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Unhandled request".to_string(),
                        request: to_json_binary(&query).unwrap(),
                    }),
                }
            }
            _ => SystemResult::Err(SystemError::InvalidRequest {
                error: "Unhandled request".to_string(),
                request: to_json_binary(&query).unwrap(),
            }),
        });

        let res = deposit_into_strategy(
            deps.as_mut(),
            info_delegation_manager.clone(),
            info_delegation_manager.sender.clone(),
            strategy.clone(),
            token.clone(),
            amount,
        )
        .unwrap();

        let non_whitelisted_strategy = deps.api.addr_make("non_whitelisted_strategy");

        let result = deposit_into_strategy(
            deps.as_mut(),
            info_delegation_manager.clone(),
            info_delegation_manager.sender.clone(),
            non_whitelisted_strategy.clone(),
            token.clone(),
            amount,
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::NotWhitelisted {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_get_deposits() {
        let (mut deps, env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker.clone(),
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100))
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200))
            .unwrap();

        // Query deposits for the staker
        let query_msg = QueryMsg::StakerDepositList {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: StakerDepositListResponse = from_json(bin).unwrap();

        assert_eq!(
            response.0,
            vec![
                StrategyShare {
                    strategy: strategy1,
                    shares: Uint128::new(100)
                },
                StrategyShare {
                    strategy: strategy2,
                    shares: Uint128::new(200)
                }
            ]
        );

        // Test with a staker that has no deposits
        let new_staker = deps.api.addr_make("new_staker").to_string();

        let query_msg = QueryMsg::StakerDepositList { staker: new_staker };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let response: StakerDepositListResponse = from_json(bin).unwrap();

        assert_eq!(response.0.len(), 0);
    }

    #[test]
    fn test_add_shares_internal() {
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = Addr::unchecked("staker");
        let strategy = Addr::unchecked("strategy");
        let shares = Uint128::new(100);

        let res =
            add_shares_internal(deps.as_mut(), staker.clone(), strategy.clone(), shares).unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);

        let additional_shares = Uint128::new(50);
        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            additional_shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, additional_shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);

        // Test with zero shares
        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            Uint128::zero(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test exceeding the max strategy list length
        let mut strategy_list = Vec::new();
        for i in 0..MAX_STRATEGY_LENGTH {
            strategy_list.push(Addr::unchecked(format!("strategy{}", i)));
        }
        STAKER_STRATEGY_LIST
            .save(&mut deps.storage, &staker, &strategy_list)
            .unwrap();

        let new_strategy = Addr::unchecked("new_strategy");
        let result =
            add_shares_internal(deps.as_mut(), staker.clone(), new_strategy.clone(), shares);
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MaxStrategyListLengthExceeded {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_add_shares() {
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let strategy = deps.api.addr_make("strategy");
        let shares = Uint128::new(100);

        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after first addition: {}", stored_shares);
        assert_eq!(stored_shares, shares);

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        assert_eq!(strategy_list.len(), 1);
        assert_eq!(strategy_list[0], strategy);

        // Test adding more shares to the same strategy
        let additional_shares = Uint128::new(50);

        let res = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            additional_shares,
        )
        .unwrap();

        let events = res.events;
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.ty, "add_shares");
        assert_eq!(event.attributes.len(), 3);
        assert_eq!(event.attributes[0].key, "staker");
        assert_eq!(event.attributes[0].value, staker.to_string());
        assert_eq!(event.attributes[1].key, "strategy");
        assert_eq!(event.attributes[1].value, strategy.to_string());
        assert_eq!(event.attributes[2].key, "shares");
        assert_eq!(event.attributes[2].value, additional_shares.to_string());

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy))
            .unwrap();
        println!("stored_shares after second addition: {}", stored_shares);
        assert_eq!(stored_shares, shares + additional_shares);

        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = add_shares(
            deps.as_mut(),
            info_unauthorized.clone(),
            staker.clone(),
            strategy.clone(),
            shares,
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy.clone(),
            Uint128::zero(),
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test exceeding the max strategy list length
        let mut strategy_list = Vec::new();
        for i in 0..MAX_STRATEGY_LENGTH {
            strategy_list.push(Addr::unchecked(format!("strategy{}", i)));
        }
        STAKER_STRATEGY_LIST
            .save(&mut deps.storage, &staker, &strategy_list)
            .unwrap();

        let new_strategy = deps.api.addr_make("new_strategy");

        let result = add_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            new_strategy.clone(),
            shares,
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::MaxStrategyListLengthExceeded {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_remove_shares() {
        let (mut deps, _env, _owner_info, info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker");
        let strategy1 = deps.api.addr_make("strategy1");
        let strategy2 = deps.api.addr_make("strategy2");

        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker,
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100))
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200))
            .unwrap();

        let res = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].key, "method");
        assert_eq!(res.attributes[0].value, "remove_shares");
        assert_eq!(res.attributes[1].key, "staker");
        assert_eq!(res.attributes[1].value, staker.to_string());
        assert_eq!(res.attributes[2].key, "strategy");
        assert_eq!(res.attributes[2].value, strategy1.to_string());
        assert_eq!(res.attributes[3].key, "shares");
        assert_eq!(res.attributes[3].value, "50");
        assert_eq!(res.attributes[4].key, "strategy_removed");
        assert_eq!(res.attributes[4].value, "false");

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy1))
            .unwrap();
        println!("Stored shares after removal: {}", stored_shares);
        assert_eq!(stored_shares, Uint128::new(50));

        // Test removing shares with an unauthorized user
        let info_unauthorized = message_info(&Addr::unchecked("unauthorized"), &[]);

        let result = remove_shares(
            deps.as_mut(),
            info_unauthorized.clone(),
            staker.clone(),
            strategy2.clone(),
            Uint128::new(50),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::Unauthorized {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test removing more shares than available
        let result = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(60),
        );

        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        // Test removing all shares, which should remove the strategy from the staker's list
        let res = remove_shares(
            deps.as_mut(),
            info_delegation_manager.clone(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[4].key, "strategy_removed");
        assert_eq!(res.attributes[4].value, "true");

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        println!("Strategy list after removal: {:?}", strategy_list);
        assert_eq!(strategy_list.len(), 1);
        assert!(!strategy_list.contains(&strategy1));
        assert!(strategy_list.contains(&strategy2));
    }

    #[test]
    fn test_remove_shares_internal() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = Addr::unchecked("staker1");
        let strategy1 = Addr::unchecked("strategy1");
        let strategy2 = Addr::unchecked("strategy2");

        STAKER_STRATEGY_LIST
            .save(
                &mut deps.storage,
                &staker,
                &vec![strategy1.clone(), strategy2.clone()],
            )
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy1), &Uint128::new(100))
            .unwrap();
        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy2), &Uint128::new(200))
            .unwrap();

        let result = remove_shares_internal(
            deps.as_mut(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();
        assert!(!result);

        let stored_shares = STAKER_STRATEGY_SHARES
            .load(&deps.storage, (&staker, &strategy1))
            .unwrap();
        println!("Stored shares after partial removal: {}", stored_shares);

        assert_eq!(stored_shares, Uint128::new(50));

        let result = remove_shares_internal(
            deps.as_mut(),
            staker.clone(),
            strategy1.clone(),
            Uint128::new(50),
        )
        .unwrap();

        assert!(result);

        let strategy_list = STAKER_STRATEGY_LIST.load(&deps.storage, &staker).unwrap();
        println!("Strategy list after full removal: {:?}", strategy_list);
        assert_eq!(strategy_list.len(), 1);
        assert!(!strategy_list.contains(&strategy1));
        assert!(strategy_list.contains(&strategy2));

        let result = remove_shares_internal(
            deps.as_mut(),
            staker.clone(),
            strategy2.clone(),
            Uint128::new(300),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }

        let result = remove_shares_internal(
            deps.as_mut(),
            staker.clone(),
            strategy2.clone(),
            Uint128::zero(),
        );
        assert!(result.is_err());
        if let Err(err) = result {
            match err {
                ContractError::InvalidShares {} => (),
                _ => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    #[test]
    fn test_get_staker_strategy_list() {
        let (mut deps, env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = deps.api.addr_make("staker1");

        let strategies = vec![
            deps.api.addr_make("strategy1"),
            deps.api.addr_make("strategy2"),
        ];
        STAKER_STRATEGY_LIST
            .save(&mut deps.storage, &staker, &strategies.clone())
            .unwrap();

        let query_msg = QueryMsg::StakerStrategyList {
            staker: staker.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let strategy_list_response: StakerStrategyListResponse = from_json(bin).unwrap();
        assert_eq!(strategy_list_response.0, strategies);

        let new_staker = deps.api.addr_make("new_staker");

        let query_msg = QueryMsg::StakerStrategyList {
            staker: new_staker.to_string(),
        };
        let bin = query(deps.as_ref(), env, query_msg).unwrap();
        let strategy_list_response: StakerStrategyListResponse = from_json(bin).unwrap();
        assert!(strategy_list_response.0.is_empty());
    }

    #[test]
    fn test_get_staker_strategy_shares() {
        let (mut deps, _env, _owner_info, _info_delegation_manager) = instantiate_contract();

        let staker = Addr::unchecked("staker1");
        let strategy = deps.api.addr_make("strategy");
        let shares = Uint128::new(100);

        STAKER_STRATEGY_SHARES
            .save(&mut deps.storage, (&staker, &strategy), &shares)
            .unwrap();

        let retrieved_shares =
            query::staker_strategy_shares(deps.as_ref(), staker.clone(), strategy.clone()).unwrap();
        assert_eq!(retrieved_shares.0, shares);

        let new_staker = Addr::unchecked("new_staker");
        let retrieved_shares =
            query::staker_strategy_shares(deps.as_ref(), new_staker.clone(), strategy.clone())
                .unwrap();
        assert_eq!(retrieved_shares.0, Uint128::zero());

        let new_strategy = Addr::unchecked("new_strategy");
        let retrieved_shares =
            query::staker_strategy_shares(deps.as_ref(), staker.clone(), new_strategy.clone())
                .unwrap();
        assert_eq!(retrieved_shares.0, Uint128::zero());
    }
}
