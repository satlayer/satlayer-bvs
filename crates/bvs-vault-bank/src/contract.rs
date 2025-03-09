#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::bank;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

const CONTRACT_NAME: &str = concat!("crate:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    bank::set_denom(deps.storage, msg.denom)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let router = deps.api.addr_validate(&msg.router)?;
    bvs_vault_base::router::set_router(deps.storage, &router)?;
    let operator = deps.api.addr_validate(&msg.operator)?;
    bvs_vault_base::router::set_operator(deps.storage, &operator)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("pauser", pauser)
        .add_attribute("router", router)
        .add_attribute("operator", operator))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_pauser::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;

    match msg {
        ExecuteMsg::Deposit(msg) => {
            msg.validate(deps.api)?;
            execute::deposit(deps, env, info, msg)
        }
        ExecuteMsg::Withdraw(msg) => {
            msg.validate(deps.api)?;
            execute::withdraw(deps, env, info, msg)
        }
    }
}

mod execute {
    use crate::bank;
    use crate::bank::get_denom;
    use crate::error::ContractError;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::RecipientAmount;
    use bvs_vault_base::{offset, router, shares};
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, StdError};

    pub fn deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;
        // Determine and compare the assets to be deposited from `info.funds` and `msg.amount`
        let new_assets = {
            let denom = get_denom(deps.storage)?;
            let amount = cw_utils::must_pay(&info, denom.as_str())?;
            if amount != msg.amount {
                return Err(
                    VaultError::insufficient("payable amount does not match msg.amount").into(),
                );
            }
            amount
        };
        let (vault, new_shares) = {
            // Bank balance is after deposit, we need to calculate the balance before deposit
            let after_balance = bank::query_balance(&deps.as_ref(), &env)?;
            let before_balance = after_balance
                .checked_sub(new_assets)
                .map_err(StdError::from)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), before_balance)?;

            let new_shares = vault.assets_to_shares(new_assets)?;
            vault.checked_add_shares(deps.storage, new_shares)?;

            (vault, new_shares)
        };

        // Add shares to the recipient
        shares::add_shares(deps.storage, &msg.recipient, new_shares)?;

        Ok(Response::new().add_event(
            Event::new("Deposit")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("recipient", msg.recipient)
                .add_attribute("assets", new_assets.to_string())
                .add_attribute("shares", new_shares.to_string())
                .add_attribute("total_shares", vault.total_shares().to_string()),
        ))
    }

    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_not_validating(&deps.as_ref())?;
        shares::sub_shares(deps.storage, &info.sender, msg.amount)?;

        let (vault, claim_assets) = {
            let balance = bank::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            if msg.amount > vault.total_shares() {
                return Err(VaultError::insufficient("Insufficient shares to withdraw.").into());
            }

            let assets = vault.shares_to_assets(msg.amount)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            vault.checked_sub_shares(deps.storage, msg.amount)?;

            (vault, assets)
        };

        // Setup transfer to recipient
        let send_msg = bank::bank_send(deps.storage, &msg.recipient, claim_assets)?;

        Ok(Response::new()
            .add_event(
                Event::new("Withdraw")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient.to_string())
                    .add_attribute("assets", claim_assets.to_string())
                    .add_attribute("shares", msg.amount.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(send_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Shares { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&query::shares(deps, staker)?)
        }
        QueryMsg::Assets { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&query::assets(deps, env, staker)?)
        }
        QueryMsg::ConvertToAssets { shares } => {
            to_json_binary(&query::convert_to_assets(deps, env, shares)?)
        }
        QueryMsg::ConvertToShares { assets } => {
            to_json_binary(&query::convert_to_shares(deps, env, assets)?)
        }
        QueryMsg::TotalShares {} => to_json_binary(&query::total_shares(deps, env)?),
        QueryMsg::TotalAssets {} => to_json_binary(&query::total_assets(deps, env)?),
        QueryMsg::VaultInfo {} => to_json_binary(&query::vault_info(deps, env)?),
    }
}

mod query {
    use crate::bank;
    use bvs_vault_base::msg::VaultInfoResponse;
    use bvs_vault_base::{offset, shares};
    use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

    pub fn shares(deps: Deps, staker: Addr) -> StdResult<Uint128> {
        shares::get_shares(deps.storage, &staker)
    }

    pub fn assets(deps: Deps, env: Env, staker: Addr) -> StdResult<Uint128> {
        let shares = shares(deps, staker)?;
        convert_to_assets(deps, env, shares)
    }

    pub fn convert_to_assets(deps: Deps, env: Env, shares: Uint128) -> StdResult<Uint128> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.shares_to_assets(shares)
    }

    pub fn convert_to_shares(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.assets_to_shares(assets)
    }

    /// Total issued shares in this vault.
    pub fn total_shares(deps: Deps, _env: Env) -> StdResult<Uint128> {
        offset::get_total_shares(deps.storage)
    }

    /// Total assets in this vault, the "asset staked" to this vault.
    pub fn total_assets(deps: Deps, env: Env) -> StdResult<Uint128> {
        bank::query_balance(&deps, &env)
    }

    /// Returns the vault information
    pub fn vault_info(deps: Deps, env: Env) -> StdResult<VaultInfoResponse> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        let denom = bank::get_denom(deps.storage)?;
        let version = cw2::get_contract_version(deps.storage)?;
        Ok(VaultInfoResponse {
            total_shares: vault.total_shares(),
            total_assets: vault.total_assets(),
            router: bvs_vault_base::router::get_router(deps.storage)?,
            pauser: bvs_pauser::api::get_pauser(deps.storage)?,
            operator: bvs_vault_base::router::get_operator(deps.storage)?,
            slashing: false,
            asset_id: format!("cosmos:{}/bank:{}", env.block.chain_id, denom.as_str()),
            contract: version.contract,
            version: version.version,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::bank;
    use crate::contract::{execute, instantiate};
    use crate::msg::InstantiateMsg;
    use bvs_vault_base::msg::RecipientAmount;
    use bvs_vault_base::{offset, router, shares};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        coins, to_json_binary, BankMsg, Coin, ContractResult, CosmosMsg, Event, Response,
        SystemError, SystemResult, Uint128, WasmQuery,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let pauser = deps.api.addr_make("pauser");
        let router = deps.api.addr_make("vault_router");
        let operator = deps.api.addr_make("operator");

        let msg = InstantiateMsg {
            pauser: pauser.to_string(),
            router: router.to_string(),
            operator: operator.to_string(),
            denom: "test".to_string(),
        };

        let info = message_info(&sender, &[]);
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("pauser", pauser)
                .add_attribute("router", router)
                .add_attribute("operator", operator)
        );
    }

    #[test]
    fn test_deposit() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");

        {
            // For QueryMsg::IsWhitelisted(vault) = true
            {
                let router = deps.api.addr_make("vault_router");
                router::set_router(&mut deps.storage, &router).unwrap();
                deps.querier.update_wasm(move |query| match query {
                    WasmQuery::Smart { .. } => {
                        return SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&true).unwrap(),
                        ));
                    }
                    _ => SystemResult::Err(SystemError::Unknown {}),
                });
            }

            bank::set_denom(&mut deps.storage, "stone").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(10_000, "stone"));
        }

        let info = message_info(&sender, &coins(10000, "stone"));
        let response = execute::deposit(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            RecipientAmount {
                recipient: sender.clone(),
                amount: Uint128::new(10_000),
            },
        )
        .unwrap();

        assert_eq!(
            response,
            Response::new().add_event(
                Event::new("Deposit")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", info.sender.to_string())
                    .add_attribute("assets", "10000")
                    .add_attribute("shares", "10000")
                    .add_attribute("total_shares", "10000"),
            )
        );

        let total_shares = offset::get_total_shares(&deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(10_000));
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");

        {
            // For QueryMsg::IsValidating(operator) = false
            {
                let router = deps.api.addr_make("vault_router");
                router::set_router(&mut deps.storage, &router).unwrap();
                let operator = deps.api.addr_make("operator");
                router::set_operator(&mut deps.storage, &operator).unwrap();
                deps.querier.update_wasm(move |query| match query {
                    WasmQuery::Smart { .. } => {
                        return SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&false).unwrap(),
                        ));
                    }
                    _ => SystemResult::Err(SystemError::Unknown {}),
                });
            }

            bank::set_denom(&mut deps.storage, "knife").unwrap();
            let balance = coins(10_000, "knife");
            deps.querier
                .bank
                .update_balance(env.contract.address.clone(), balance);

            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), Uint128::zero()).unwrap();
            vault
                .checked_add_shares(&mut deps.storage, Uint128::new(10_000))
                .unwrap();
            shares::add_shares(&mut deps.storage, &sender, Uint128::new(10_000)).unwrap();
        }

        let recipient = deps.api.addr_make("recipient");
        let sender_info = message_info(&sender, &[]);
        let response = execute::withdraw(
            deps.as_mut(),
            env.clone(),
            sender_info.clone(),
            RecipientAmount {
                recipient: recipient.clone(),
                amount: Uint128::new(10_000),
            },
        )
        .unwrap();

        assert_eq!(
            response,
            Response::new()
                .add_event(
                    Event::new("Withdraw")
                        .add_attribute("sender", sender.to_string())
                        .add_attribute("recipient", recipient.to_string())
                        .add_attribute("assets", "10000")
                        .add_attribute("shares", "10000")
                        .add_attribute("total_shares", "0")
                )
                .add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: "knife".to_string(),
                        amount: Uint128::new(10_000)
                    }],
                }))
        );

        let total_shares = offset::get_total_shares(&deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::zero());
    }
}
