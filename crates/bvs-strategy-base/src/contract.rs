#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    auth,
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    shares, token,
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use bvs_library::ownership;

const CONTRACT_NAME: &str = "BVS Strategy Base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;
    auth::set_strategy_manager(deps.storage, &strategy_manager)?;

    let registry = deps.api.addr_validate(&msg.registry)?;
    bvs_registry::api::set_registry_addr(deps.storage, &registry)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let underlying_token = deps.api.addr_validate(&msg.underlying_token)?;
    token::set_cw20_token(deps.storage, &underlying_token)?;

    // Query the underlying token to ensure the token has TokenInfo entry_point
    token::get_token_info(&deps.as_ref())?;

    shares::set_total_shares(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("strategy_manager", strategy_manager)
        // TODO(fuxingloh): rename to asset
        .add_attribute("underlying_token", underlying_token))
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
        ExecuteMsg::Deposit { amount } => execute::deposit(deps, env, info, amount),
        ExecuteMsg::Withdraw { recipient, shares } => {
            let recipient = deps.api.addr_validate(&recipient)?;
            execute::withdraw(deps, env, info, recipient, shares)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
    }
}

pub mod execute {
    use crate::{auth, shares, token, ContractError};
    use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response, Uint128};

    /// Deposit tokens into the strategy.
    /// Token with a fees-on-transfer model is not supported and will break the exchange rate.
    /// This is mitigated,
    /// as strategies (and the underlying CW20 tokens)
    /// are whitelisted to ensure that the token does not have fees-on-transfer.
    pub fn deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        auth::assert_strategy_manager(deps.storage, &info)?;

        let virtual_share = shares::VirtualVault::load(&deps.as_ref(), &env)?;
        let new_shares = virtual_share.amount_to_shares(amount);

        if new_shares.is_zero() {
            return Err(ContractError::zero("New shares cannot be zero."));
        }

        let mut total_shares = virtual_share.total_shares;
        total_shares += new_shares;
        shares::set_total_shares(deps.storage, &total_shares)?;

        // TODO(fuxingloh): sub_messages
        // let transfer_from_msg = token::new_transfer_from(
        //     &deps.as_ref(),
        //     &owner,
        //     &env.contract.address,
        //     amount,
        // )?;

        Ok(Response::new().add_event(
            Event::new("Deposit")
                // TODO(fuxingloh): add owner
                .add_attribute("amount", amount.to_string())
                .add_attribute("shares", new_shares.to_string())
                .add_attribute("total_shares", total_shares.to_string()),
        ))
    }

    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: Addr,
        shares: Uint128,
    ) -> Result<Response, ContractError> {
        auth::assert_strategy_manager(deps.storage, &info)?;

        let virtual_share = shares::VirtualVault::load(&deps.as_ref(), &env)?;

        let mut total_shares = virtual_share.total_shares;
        if shares > total_shares {
            return Err(ContractError::insufficient(
                "Insufficient shares to withdraw.",
            ));
        }

        let amount = virtual_share.shares_to_amount(shares);
        if amount.is_zero() {
            return Err(ContractError::zero("Amount cannot be zero."));
        }

        if amount > virtual_share.balance {
            return Err(ContractError::insufficient(
                "Insufficient balance to withdraw.",
            ));
        }

        total_shares -= shares;
        shares::set_total_shares(deps.storage, &total_shares)?;

        // Setup transfer to recipient
        let transfer_msg = token::new_transfer(deps.storage, &recipient, amount)?;

        Ok(Response::new()
            .add_event(
                Event::new("Withdraw")
                    .add_attribute("recipient", amount.to_string())
                    .add_attribute("amount", amount.to_string())
                    .add_attribute("shares", shares.to_string())
                    .add_attribute("total_shares", total_shares.to_string()),
            )
            .add_message(transfer_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Shares { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&query::shares(deps, &env, staker)?)
        }
        QueryMsg::Underlying { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&query::underlying(deps, &env, staker)?)
        }
        QueryMsg::SharesToUnderlying { shares } => {
            to_json_binary(&query::shares_to_underlying(deps, &env, shares)?)
        }
        QueryMsg::UnderlyingToShares { amount } => {
            to_json_binary(&query::underlying_to_shares(deps, &env, amount)?)
        }
        QueryMsg::StrategyManager {} => to_json_binary(&query::strategy_manager(deps)?),
        QueryMsg::UnderlyingToken {} => to_json_binary(&query::underlying_token(deps)?),
        QueryMsg::TotalShares {} => to_json_binary(&query::total_shares(deps)?),
    }
}

pub mod query {
    use crate::msg::{
        SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse, TotalSharesResponse,
        UnderlyingResponse, UnderlyingToSharesResponse, UnderlyingTokenResponse,
    };
    use crate::{auth, shares, token};
    use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

    /// Returns the amount of shares held by the staker.
    /// Information is sourced from the strategy manager.
    pub fn shares(deps: Deps, env: &Env, staker: Addr) -> StdResult<SharesResponse> {
        let strategy_manager = auth::get_strategy_manager(deps.storage)?;
        let strategy = env.contract.address.to_string();

        let shares = crate::msg::strategy_manager::get_staker_strategy_shares(
            &deps.querier,
            strategy_manager.to_string(),
            strategy.to_string(),
            staker.to_string(),
        )?;

        Ok(SharesResponse(shares))
    }

    /// Returns the amount of underlying tokens held by the staker.
    /// Information is sourced from the strategy manager,
    /// by converting total shares held by the staker to underlying tokens.
    /// TODO(fuxingloh): rename `assets`
    pub fn underlying(deps: Deps, env: &Env, staker: Addr) -> StdResult<UnderlyingResponse> {
        let SharesResponse(shares) = shares(deps, env, staker)?;
        let SharesToUnderlyingResponse(amount) = shares_to_underlying(deps, env, shares)?;
        Ok(UnderlyingResponse(amount))
    }

    /// Converts the amount of shares to underlying tokens.
    /// See [`shares::VirtualVault`] implementation for more details.
    /// TODO(fuxingloh): rename `convert_to_assets`
    pub fn shares_to_underlying(
        deps: Deps,
        env: &Env,
        shares: Uint128,
    ) -> StdResult<SharesToUnderlyingResponse> {
        let vault = shares::VirtualVault::load(&deps, env)?;
        let amount = vault.shares_to_amount(shares);
        Ok(SharesToUnderlyingResponse(amount))
    }

    /// Converts the amount of underlying tokens to shares.
    /// See [`shares::VirtualVault`] implementation for more details.
    /// TODO(fuxingloh): rename `convert_to_shares`
    pub fn underlying_to_shares(
        deps: Deps,
        env: &Env,
        amount: Uint128,
    ) -> StdResult<UnderlyingToSharesResponse> {
        let vault = shares::VirtualVault::load(&deps, env)?;
        let shares = vault.amount_to_shares(amount);
        Ok(UnderlyingToSharesResponse(shares))
    }

    /// Returns the strategy manager address.
    pub fn strategy_manager(deps: Deps) -> StdResult<StrategyManagerResponse> {
        let strategy_manager = auth::get_strategy_manager(deps.storage)?;
        Ok(StrategyManagerResponse(strategy_manager))
    }

    // TODO(fuxingloh): add `total_assets`

    /// Returns the underlying token address.
    /// TODO(fuxingloh): rename `asset_info` (similar to Cw20QueryMsg::AssetInfo)
    pub fn underlying_token(deps: Deps) -> StdResult<UnderlyingTokenResponse> {
        let underlying_token = token::get_c20_token(deps.storage)?;
        Ok(UnderlyingTokenResponse(underlying_token))
    }

    /// Returns the total shares in the strategy.
    pub fn total_shares(deps: Deps) -> StdResult<TotalSharesResponse> {
        let shares = shares::get_total_shares(deps.storage)?;
        Ok(TotalSharesResponse(shares))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{
        SharesResponse, SharesToUnderlyingResponse, StrategyManagerResponse, TotalSharesResponse,
        UnderlyingResponse, UnderlyingToSharesResponse, UnderlyingTokenResponse,
    };
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{from_json, ContractResult, Event, SystemError, SystemResult, WasmQuery};
    use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
            strategy_manager: strategy_manager.to_string(),
            underlying_token: token.to_string(),
        };

        deps.querier.update_wasm(move |query| match query {
            WasmQuery::Smart { .. } => {
                return SystemResult::Ok(ContractResult::Ok(
                    to_json_binary(&TokenInfoResponse {
                        name: "Mock Token".to_string(),
                        symbol: "MTK".to_string(),
                        decimals: 8,
                        total_supply: Uint128::new(1_000_000),
                    })
                    .unwrap(),
                ));
            }
            _ => SystemResult::Err(SystemError::Unknown {}),
        });

        let info = message_info(&owner, &[]);
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("strategy_manager", strategy_manager)
                .add_attribute("underlying_token", token)
        );
    }

    #[test]
    fn test_deposit() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::zero()).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(0),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let amount = Uint128::new(10_000);
        let info = message_info(&strategy_manager, &[]);
        let response = execute::deposit(deps.as_mut(), env.clone(), info.clone(), amount).unwrap();
        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("Deposit")
                    .add_attribute("amount", "10000")
                    .add_attribute("shares", "10000")
                    .add_attribute("total_shares", "10000")
            )
        );

        let total_shares = shares::get_total_shares(&mut deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(10_000));
    }

    #[test]
    fn test_deposit_inflation() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(1)).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(100_000),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let amount = Uint128::new(5_912);
        let info = message_info(&strategy_manager, &[]);
        let response = execute::deposit(deps.as_mut(), env.clone(), info.clone(), amount).unwrap();
        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("Deposit")
                    .add_attribute("amount", "5912")
                    .add_attribute("shares", "58")
                    .add_attribute("total_shares", "59")
            )
        );

        let total_shares = shares::get_total_shares(&mut deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(59));
    }

    #[test]
    fn test_deposit_200_to_100() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(100)).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(200),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let amount = Uint128::new(9631);
        let info = message_info(&strategy_manager, &[]);
        let response = execute::deposit(deps.as_mut(), env.clone(), info.clone(), amount).unwrap();
        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("Deposit")
                    .add_attribute("amount", "9631")
                    .add_attribute("shares", "8828")
                    .add_attribute("total_shares", "8928")
            )
        );

        let total_shares = shares::get_total_shares(&mut deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(8928));
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(10_000)).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(10_000),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let amount = Uint128::new(10_000);
        let recipient = deps.api.addr_make("recipient");
        let info = message_info(&strategy_manager, &[]);
        let response = execute::withdraw(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            recipient.clone(),
            amount,
        )
        .unwrap();

        assert_eq!(
            response,
            Response::new()
                .add_event(
                    Event::new("Withdraw")
                        .add_attribute("recipient", "10000")
                        .add_attribute("amount", "10000")
                        .add_attribute("shares", "10000")
                        .add_attribute("total_shares", "0")
                )
                .add_message(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: token.to_string(),
                    msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: recipient.to_string(),
                        amount: Uint128::new(10_000),
                    })
                    .unwrap(),
                    funds: vec![],
                })
        );

        let total_shares = shares::get_total_shares(&mut deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(0));
    }

    // TODO: need more deposit/withdraw tests

    #[test]
    fn test_query_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } if contract_addr == &strategy_manager.to_string() => {
                    return match from_json::<crate::msg::strategy_manager::QueryMsg>(msg).unwrap() {
                        crate::msg::strategy_manager::QueryMsg::GetStakerStrategyShares {
                            ..
                        } => SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(
                                &crate::msg::strategy_manager::StakerStrategySharesResponse {
                                    shares: Uint128::new(404),
                                },
                            )
                            .unwrap(),
                        )),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let staker = deps.api.addr_make("staker");
        let SharesResponse(shares) = query::shares(deps.as_ref(), &env, staker).unwrap();
        assert_eq!(shares, Uint128::new(404));
    }

    #[test]
    fn test_query_underlying() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(10_000)).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } if contract_addr == &strategy_manager.to_string() => {
                    return match from_json::<crate::msg::strategy_manager::QueryMsg>(msg).unwrap() {
                        crate::msg::strategy_manager::QueryMsg::GetStakerStrategyShares {
                            ..
                        } => SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(
                                &crate::msg::strategy_manager::StakerStrategySharesResponse {
                                    shares: Uint128::new(404),
                                },
                            )
                            .unwrap(),
                        )),
                    }
                }
                WasmQuery::Smart {
                    contract_addr, msg, ..
                } if contract_addr == &token.to_string() => {
                    return match from_json::<Cw20QueryMsg>(msg).unwrap() {
                        Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&BalanceResponse {
                                balance: Uint128::new(100_000),
                            })
                            .unwrap(),
                        )),
                        _ => SystemResult::Err(SystemError::Unknown {}),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let staker = deps.api.addr_make("staker");
        let UnderlyingResponse(underlying) =
            query::underlying(deps.as_ref(), &env, staker).unwrap();

        // (Balance + OFFSET) / (TotalShares + OFFSET) * Shares
        // (100,000 + 1,000) / (10,000 + 1,000) * 404 = 3,709.45
        assert_eq!(underlying, Uint128::new(3709));
    }

    #[test]
    fn test_shares_to_underlying() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(10_000)).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(10_500),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        let SharesToUnderlyingResponse(amount) =
            query::shares_to_underlying(deps.as_ref(), &env, Uint128::new(999)).unwrap();

        // (10,500 + 1,000) / (10,000 + 1,000) * 999 = 1,044.40
        assert_eq!(amount, Uint128::new(1044));
    }

    #[test]
    fn test_underlying_to_shares() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let strategy_manager = deps.api.addr_make("strategy_manager");
        let token = deps.api.addr_make("token");

        {
            auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();
            token::set_cw20_token(&mut deps.storage, &token).unwrap();
            shares::set_total_shares(&mut deps.storage, &Uint128::new(782_367_326_939_736))
                .unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart { msg, .. } => match from_json::<Cw20QueryMsg>(msg).unwrap() {
                    Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&BalanceResponse {
                            balance: Uint128::new(799_555_143_531_452),
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                },
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        // (Total Shares + Offset) / (Balance + Offset) * Amount
        // You get lesser shares cause the balance is higher
        // (782,367,326,939,736 + 1,000) / (799,555,143,531,452 + 1,000) * 5,000,000 = 4,892,516.37
        let UnderlyingToSharesResponse(shares) =
            query::underlying_to_shares(deps.as_ref(), &env, Uint128::new(5_000_000)).unwrap();
        assert_eq!(shares, Uint128::new(4_892_516));

        // You get back the same amount you started with, -1 due to rounding
        let SharesToUnderlyingResponse(amount) =
            query::shares_to_underlying(deps.as_ref(), &env, Uint128::new(4_892_516)).unwrap();
        assert_eq!(amount, Uint128::new(4_999_999));
    }

    #[test]
    fn test_strategy_manager() {
        let mut deps = mock_dependencies();
        let strategy_manager = deps.api.addr_make("strategy_manager");

        auth::set_strategy_manager(&mut deps.storage, &strategy_manager).unwrap();

        let StrategyManagerResponse(addr) = query::strategy_manager(deps.as_ref()).unwrap();
        assert_eq!(addr, strategy_manager);
    }

    #[test]
    fn test_underlying_token() {
        let mut deps = mock_dependencies();
        let token = deps.api.addr_make("token");

        token::set_cw20_token(&mut deps.storage, &token).unwrap();

        let UnderlyingTokenResponse(addr) = query::underlying_token(deps.as_ref()).unwrap();
        assert_eq!(addr, token);
    }

    #[test]
    fn test_total_shares() {
        let mut deps = mock_dependencies();

        shares::set_total_shares(&mut deps.storage, &Uint128::new(10_000)).unwrap();

        let TotalSharesResponse(shares) = query::total_shares(deps.as_ref()).unwrap();
        assert_eq!(shares, Uint128::new(10_000));
    }
}
