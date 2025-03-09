#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::token;
use crate::token::get_token_info;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crate:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let router = deps.api.addr_validate(&msg.router)?;
    bvs_vault_base::router::set_router(deps.storage, &router)?;
    let operator = deps.api.addr_validate(&msg.operator)?;
    bvs_vault_base::router::set_operator(deps.storage, &operator)?;

    let cw20_contract = deps.api.addr_validate(&msg.cw20_contract)?;
    token::instantiate(deps.storage, &cw20_contract)?;

    // Assert that the contract is able
    // to query the token info to ensure that the contract is properly set up
    get_token_info(&deps.as_ref())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("pauser", pauser)
        .add_attribute("router", router)
        .add_attribute("operator", operator)
        .add_attribute("cw20_contract", cw20_contract))
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
    use crate::error::ContractError;
    use crate::token;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::RecipientAmount;
    use bvs_vault_base::{offset, router, shares};
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response};

    pub fn deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;
        let assets = msg.amount;
        let (virtual_offset, new_shares) = {
            let balance = token::query_balance(&deps.as_ref(), &env)?;
            let mut virtual_offset = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let new_shares = virtual_offset.assets_to_shares(assets)?;
            virtual_offset.checked_add_shares(deps.storage, new_shares)?;

            (virtual_offset, new_shares)
        };

        // Transfer CW20 from sender to contract
        let transfer_msg = token::execute_transfer_from(
            deps.storage,
            &info.sender,
            &env.contract.address,
            msg.amount,
        )?;

        // Add shares to the recipient
        shares::add_shares(deps.storage, &msg.recipient, new_shares)?;

        Ok(Response::new()
            .add_event(
                Event::new("Deposit")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient)
                    .add_attribute("assets", assets.to_string())
                    .add_attribute("shares", new_shares.to_string())
                    .add_attribute("total_shares", virtual_offset.total_shares().to_string()),
            )
            .add_message(transfer_msg))
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
            let balance = token::query_balance(&deps.as_ref(), &env)?;
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
        let transfer_msg = token::execute_new_transfer(deps.storage, &msg.recipient, claim_assets)?;

        Ok(Response::new()
            .add_event(
                Event::new("Withdraw")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient.to_string())
                    .add_attribute("assets", claim_assets.to_string())
                    .add_attribute("shares", msg.amount.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(transfer_msg))
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
    use crate::token;
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
        let balance = token::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.shares_to_assets(shares)
    }

    pub fn convert_to_shares(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let balance = token::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.assets_to_shares(assets)
    }

    /// Total issued shares in this vault.
    pub fn total_shares(deps: Deps, _env: Env) -> StdResult<Uint128> {
        offset::get_total_shares(deps.storage)
    }

    /// Total assets in this vault, the "asset staked" to this vault.
    pub fn total_assets(deps: Deps, env: Env) -> StdResult<Uint128> {
        token::query_balance(&deps, &env)
    }

    /// Returns the vault information
    pub fn vault_info(deps: Deps, env: Env) -> StdResult<VaultInfoResponse> {
        let balance = token::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        let cw20_contract = token::get_cw20_contract(deps.storage)?;
        let version = cw2::get_contract_version(deps.storage)?;
        Ok(VaultInfoResponse {
            total_shares: vault.total_shares(),
            total_assets: vault.total_assets(),
            router: bvs_vault_base::router::get_router(deps.storage)?,
            pauser: bvs_pauser::api::get_pauser(deps.storage)?,
            operator: bvs_vault_base::router::get_operator(deps.storage)?,
            slashing: false,
            asset_id: format!(
                "cosmos:{}/cw20:{}",
                env.block.chain_id,
                cw20_contract.as_str()
            ),
            contract: version.contract,
            version: version.version,
        })
    }
}

#[cfg(test)]
mod tests {}
