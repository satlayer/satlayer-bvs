use cosmwasm_std::to_json_binary;
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20_base::contract::instantiate as base_instantiate;
use cw20_base::msg::InstantiateMsg as ReceiptCw20InstantiateMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg as CombinedExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use bvs_vault_bank::bank as UnderlyingToken;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let router = deps.api.addr_validate(&msg.router)?;
    bvs_vault_base::router::set_router(deps.storage, &router)?;
    let operator = deps.api.addr_validate(&msg.operator)?;
    bvs_vault_base::router::set_operator(deps.storage, &operator)?;

    UnderlyingToken::set_denom(deps.storage, &msg.denom)?;

    let receipt_token_instantiate = ReceiptCw20InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: vec![],
        mint: None,
        marketing: None,
    };

    let mut response = base_instantiate(deps.branch(), env, info, receipt_token_instantiate)?;

    // important to set the set_contract_version after the base contract instantiation
    // because base_instantiate set the contract name and version with
    // its own hardcoded values
    // Setting again so this vault overwrites it
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // merge the base response with the custom response
    response = response
        .add_attribute("method", "instantiate")
        .add_attribute("pauser", pauser)
        .add_attribute("router", router)
        .add_attribute("operator", operator)
        .add_attribute("denom", msg.denom);

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CombinedExecuteMsg,
) -> Result<Response, ContractError> {
    bvs_pauser::api::assert_can_execute(deps.as_ref(), &env, &info, &msg)?;
    match msg {
        CombinedExecuteMsg::DepositFor(msg) => {
            msg.validate(deps.api)?;
            vault_execute::deposit_for(deps, env, info, msg)
        }
        CombinedExecuteMsg::QueueWithdrawalTo(msg) => {
            msg.validate(deps.api)?;
            vault_execute::queue_withdrawal_to(deps, env, info, msg)
        }
        CombinedExecuteMsg::RedeemWithdrawalTo(msg) => {
            msg.validate(deps.api)?;
            vault_execute::redeem_withdrawal_to(deps, env, info, msg)
        }
        CombinedExecuteMsg::SlashLocked(msg) => {
            msg.validate(deps.api)?;
            vault_execute::slash_locked(deps, env, info, msg)
        }
        CombinedExecuteMsg::SetApproveProxy(msg) => {
            msg.validate(deps.api)?;
            vault_execute::set_approve_proxy(deps, info, msg)
        }
        _ => {
            // cw20 compliant messages are passed to the `cw20-base` contract.
            // Except for the `Burn` and `BurnFrom` messages.
            receipt_cw20_execute::execute_base(deps, env, info, msg).map_err(Into::into)
        }
    }
}

/// CW20 messages (except for `Mint`, `Burn` and `BurnFrom`) are passed to the `cw20-base` contract.
/// The token total supply should be changed is through staking and unstaking.
/// and only through `deposit_for` and `redeem_withdrawal_to` messages.
mod receipt_cw20_execute {
    use cosmwasm_std::{Addr, StdError, StdResult, Uint128};
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

    use cw20_base::contract::execute_send;
    use cw20_base::contract::execute_transfer;

    use cw20_base::allowances::execute_decrease_allowance;
    use cw20_base::allowances::execute_increase_allowance;
    use cw20_base::allowances::execute_send_from;
    use cw20_base::allowances::execute_transfer_from;

    use cw20_base::state::{BALANCES as RECEIPT_TOKEN_BALANCES, TOKEN_INFO as RECEIPT_TOKEN_INFO};

    use crate::msg::ExecuteMsg as CombinedExecuteMsg;

    /// This mint function is almost identical to the base cw20 contract's mint function
    /// down to the variables and logic.
    /// Except that it does not require the caller to be the minter.
    pub fn mint_internal(
        deps: DepsMut,
        recipient: Addr,
        amount: Uint128,
    ) -> Result<Uint128, cw20_base::ContractError> {
        let mut config = RECEIPT_TOKEN_INFO
            .may_load(deps.storage)?
            .ok_or(cw20_base::ContractError::Unauthorized {})?;

        // update supply
        config.total_supply += amount;

        RECEIPT_TOKEN_INFO.save(deps.storage, &config)?;

        RECEIPT_TOKEN_BALANCES.update(
            deps.storage,
            &recipient,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;
        Ok(config.total_supply)
    }

    pub fn execute_base(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: CombinedExecuteMsg,
    ) -> Result<Response, cw20_base::ContractError> {
        match msg {
            CombinedExecuteMsg::Transfer { recipient, amount } => {
                execute_transfer(deps, env, info, recipient, amount)
            }
            CombinedExecuteMsg::Send {
                contract,
                amount,
                msg,
            } => execute_send(deps, env, info, contract, amount, msg),
            CombinedExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            } => execute_increase_allowance(deps, env, info, spender, amount, expires),
            CombinedExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
            CombinedExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => execute_transfer_from(deps, env, info, owner, recipient, amount),
            CombinedExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            } => execute_send_from(deps, env, info, owner, contract, amount, msg),
            _ => {
                // Extended execute msg set are exhausted in entry point already
                // Base cw20 execute msg are also exhausted in other match arm
                // So this means someone is trying to call a non-supported message
                Err(cw20_base::ContractError::Std(StdError::generic_err(
                    "This message is not supported",
                )))
            }
        }
    }
}

/// Additional vault logic are built on top of the base CW20 contract via an extended execute msg set.
/// The extended execute msg set is practically `bvs-vault-base` crate's execute msg set.
mod vault_execute {
    use crate::error::ContractError;
    use bvs_vault_bank::bank as UnderlyingToken;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::{
        QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams, SetApproveProxyParams,
    };
    use bvs_vault_base::{
        offset, proxy, router,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, StdError};
    use cw20_base::contract::execute_burn as receipt_token_burn;

    /// This executes a bank transfer of assets from the `info.sender` to the vault contract.
    ///
    /// New receipt token are minted, based on the exchange rate, to `msg.recipient`.  
    pub fn deposit_for(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;

        // Determine and compare the assets to be deposited from `info.funds` and `msg.amount`
        let amount_deposited = {
            let denom = UnderlyingToken::get_denom(deps.storage)?;
            let amount = cw_utils::must_pay(&info, denom.as_str())?;
            if amount != msg.amount {
                return Err(
                    VaultError::insufficient("payable amount does not match msg.amount").into(),
                );
            }
            amount
        };

        let new_receipt_tokens_to_be_mint = {
            // Bank balance is after deposit, we need to calculate the balance before deposit
            let after_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;
            let before_balance = after_balance
                .checked_sub(amount_deposited)
                .map_err(StdError::from)?;
            let total_receipt_token_supply =
                cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;
            let vault = offset::VirtualOffset::new(total_receipt_token_supply, before_balance)?;

            vault.assets_to_shares(amount_deposited)?
        };

        // critical section
        // Issue receipt token to msg.recipient
        // mint new receipt token to staker
        let total_supply = super::receipt_cw20_execute::mint_internal(
            deps.branch(),
            msg.recipient.clone(),
            new_receipt_tokens_to_be_mint,
        )?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", msg.recipient.to_string())
            .add_attribute("amount", new_receipt_tokens_to_be_mint.to_string())
            .add_event(
                Event::new("DepositFor")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient.to_string())
                    .add_attribute("assets", amount_deposited.to_string())
                    .add_attribute("shares", new_receipt_tokens_to_be_mint.to_string())
                    .add_attribute("total_shares", total_supply.to_string()),
            ))
    }

    /// Queue receipt tokens to withdraw later.
    /// The receipt tokens are ill-liquidated and wait lock period to redeem withdrawal.
    /// It doesn't impact `total_supply` and only take away the owner's receipt tokens, so the exchange rate is not affected.
    pub fn queue_withdrawal_to(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: QueueWithdrawalToParams,
    ) -> Result<Response, ContractError> {
        // check if the sender is the owner or an approved proxy
        if msg.owner != info.sender
            && !proxy::is_approved_proxy(deps.storage, &msg.owner, &info.sender)?
        {
            return Err(VaultError::unauthorized("Unauthorized sender").into());
        }

        // check if the sender is the controller or an approved proxy
        if msg.controller != info.sender
            && !proxy::is_approved_proxy(deps.storage, &msg.controller, &info.sender)?
        {
            return Err(VaultError::unauthorized("Unauthorized controller").into());
        }

        // ill-liquidate the receipt token from the owner
        // by moving the asset into this vault balance.
        // We can't burn until the actual unstaking (redeem withdrawal) occurs.
        // due to total supply mutation can impact the exchange rate to change prematurely.
        // We are not using `TransferFrom` here because either the owner or approved proxy should not deduct the allowance.
        let owner_info = MessageInfo {
            sender: msg.owner.clone(),
            funds: vec![],
        };
        cw20_base::contract::execute_transfer(
            deps.branch(),
            env.clone(),
            owner_info,
            env.contract.address.to_string(),
            msg.amount,
        )?;

        let withdrawal_lock_period: u64 =
            router::get_withdrawal_lock_period(&deps.as_ref())?.into();
        let current_timestamp = env.block.time;
        let unlock_timestamp = current_timestamp.plus_seconds(withdrawal_lock_period);

        let new_queued_withdrawal_info = QueuedWithdrawalInfo {
            queued_shares: msg.amount,
            unlock_timestamp,
        };

        let result = shares::update_queued_withdrawal_info(
            deps.storage,
            &msg.controller,
            new_queued_withdrawal_info,
        )?;

        Ok(Response::new().add_event(
            Event::new("QueueWithdrawalTo")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("owner", msg.owner.to_string())
                .add_attribute("controller", msg.controller.to_string())
                .add_attribute("queued_shares", msg.amount.to_string())
                .add_attribute(
                    "new_unlock_timestamp",
                    unlock_timestamp.seconds().to_string(),
                )
                .add_attribute("total_queued_shares", result.queued_shares.to_string()),
        ))
    }

    /// Redeem all queued shares to assets for `msg.controller`.
    /// The `info.sender` must be the `msg.controller` or an approved proxy.
    pub fn redeem_withdrawal_to(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RedeemWithdrawalToParams,
    ) -> Result<Response, ContractError> {
        // check if msg.controller is the sender or an approved proxy
        if msg.controller != info.sender
            && !proxy::is_approved_proxy(deps.storage, &msg.controller, &info.sender)?
        {
            return Err(VaultError::unauthorized("Unauthorized controller").into());
        }

        let withdrawal_info = shares::get_queued_withdrawal_info(deps.storage, &msg.controller)?;
        let queued_shares = withdrawal_info.queued_shares;
        let unlock_timestamp = withdrawal_info.unlock_timestamp;

        if queued_shares.is_zero() || unlock_timestamp.seconds() == 0 {
            return Err(VaultError::zero("No queued shares").into());
        }

        if unlock_timestamp.seconds() > env.block.time.seconds() {
            return Err(VaultError::locked("The shares are locked").into());
        }

        let claimed_assets = {
            let underlying_token_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;
            let receipt_token_supply =
                cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;
            let vault = offset::VirtualOffset::new(receipt_token_supply, underlying_token_balance)?;

            let assets = vault.shares_to_assets(queued_shares)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero").into());
            }

            assets
        };

        // bank transfer of asset to msg.recipient
        let transfer_msg =
            UnderlyingToken::bank_send(deps.storage, &msg.recipient, claimed_assets)?;

        // When staker queued the withdrawal
        // The receipt token is ill-liquidated from the staker
        // by moving the asset into this vault balance.
        // So the vault should burn from its own balance for the same amount.
        let msg_info = MessageInfo {
            sender: env.contract.address.clone(),
            funds: vec![],
        };
        receipt_token_burn(deps.branch(), env.clone(), msg_info, queued_shares)?;

        let receipt_token_supply =
            cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;

        // Remove controller's queued withdrawal info
        shares::remove_queued_withdrawal_info(deps.storage, &msg.controller);

        Ok(Response::new()
            .add_event(
                Event::new("RedeemWithdrawalTo")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("controller", msg.controller.to_string())
                    .add_attribute("recipient", msg.recipient.to_string())
                    .add_attribute("sub_shares", queued_shares.to_string())
                    .add_attribute("claimed_assets", claimed_assets.to_string())
                    .add_attribute("total_shares", receipt_token_supply.to_string()),
            )
            .add_message(transfer_msg))
    }

    /// Moves the assets from the vault to the `vault-router` contract.
    /// Part of the [https://build.satlayer.xyz/getting-started/slashing](Programmable Slashing) lifecycle.
    /// This function can only be called by `vault-router`, and takes an absolute `amount` of assets to be moved.
    /// The amount is calculated and enforced by the router.
    pub fn slash_locked(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: bvs_vault_base::msg::Amount,
    ) -> Result<Response, ContractError> {
        router::assert_router(deps.as_ref().storage, &info)?;

        // If the code above (`assert_router`) succeeds, it means the sender is the router
        // No need to load from storage.
        let router = info.sender;

        let vault_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;

        if amount.0 > vault_balance {
            return Err(VaultError::insufficient("Not enough balance").into());
        }

        let transfer_msg = UnderlyingToken::bank_send(deps.storage, &router, amount.0)?;

        let event = Event::new("SlashLocked")
            .add_attribute("sender", router.to_string())
            .add_attribute("amount", amount.0.to_string())
            .add_attribute(
                "denom",
                UnderlyingToken::get_denom(deps.storage)?.to_string(),
            );

        Ok(Response::new().add_event(event).add_message(transfer_msg))
    }

    pub fn set_approve_proxy(
        deps: DepsMut,
        info: MessageInfo,
        msg: SetApproveProxyParams,
    ) -> Result<Response, ContractError> {
        proxy::set_approved_proxy(deps.storage, &info.sender, &msg.proxy, &msg.approve)?;

        Ok(Response::new().add_event(
            Event::new("SetApproveProxy")
                .add_attribute("owner", info.sender.to_string())
                .add_attribute("proxy", msg.proxy.to_string())
                .add_attribute("approved", msg.approve.to_string()),
        ))
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<cosmwasm_std::Binary> {
    match msg {
        QueryMsg::Shares { staker } => to_json_binary(&vault_query::balance_of(deps, staker)?),
        QueryMsg::Assets { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&vault_query::assets(deps, env, staker)?)
        }
        QueryMsg::ConvertToAssets { shares } => to_json_binary(
            &vault_query::convert_to_underlying_token(deps, env, shares)?,
        ),
        QueryMsg::ConvertToShares { assets } => {
            to_json_binary(&vault_query::convert_to_receipt_token(deps, env, assets)?)
        }
        QueryMsg::TotalShares {} => {
            to_json_binary(&vault_query::total_receipt_token_supply(deps, env)?)
        }
        QueryMsg::TotalAssets {} => to_json_binary(&vault_query::total_assets(deps, env)?),
        QueryMsg::QueuedWithdrawal { controller } => {
            let controller = deps.api.addr_validate(&controller)?;
            to_json_binary(&vault_query::queued_withdrawal(deps, controller)?)
        }
        QueryMsg::VaultInfo {} => to_json_binary(&vault_query::vault_info(deps, env)?),
        _ => {
            // cw20 compliant messages are passed to the `cw20-base` contract.
            cw20_base::contract::query(deps, env, msg.try_into().unwrap())
        }
    }
}

mod vault_query {
    use bvs_vault_bank::bank as UnderlyingToken;
    use bvs_vault_base::msg::{AssetType, VaultInfoResponse};
    use bvs_vault_base::{
        offset,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
    use cw20_base::contract::query_balance;

    /// Get receipt token balance of the staker
    /// Since this vault is tokenized, shares are practically the receipt token.
    /// Such that querying shares is equivalent to querying the receipt token balance of a
    /// particular staker/address.
    /// But we will support this query to keep the API consistent with the non-tokenized vault.
    /// This helps with the contract consumer/frontend to minimize code changes.
    pub fn balance_of(deps: Deps, staker: String) -> StdResult<Uint128> {
        // this func come from the cw20_base crate
        // validate the staker address
        let balance = query_balance(deps, staker)?;

        StdResult::Ok(balance.balance)
    }

    /// Get the staking token of staker, converted from receipt_tokens held by staker.
    pub fn assets(deps: Deps, env: Env, staker: Addr) -> StdResult<Uint128> {
        let balance = query_balance(deps, staker.to_string())?;
        convert_to_underlying_token(deps, env, balance.balance)
    }

    /// Given the number of receipt_token, convert to staking token based on the vault exchange rate.
    pub fn convert_to_underlying_token(
        deps: Deps,
        env: Env,
        receipt_tokens: Uint128,
    ) -> StdResult<Uint128> {
        let underlying_token_balance = UnderlyingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let vault = offset::VirtualOffset::new(receipt_token_supply, underlying_token_balance)?;
        vault.shares_to_assets(receipt_tokens)
    }

    /// Given assets, get the resulting receipt tokens based on the vault exchange rate.
    /// Shares in this tokenized vault the receipt token.
    /// Keeping the msg name the same as the non-tokenized vault for consistency.
    pub fn convert_to_receipt_token(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let underlying_token_balance = UnderlyingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let vault = offset::VirtualOffset::new(receipt_token_supply, underlying_token_balance)?;
        vault.assets_to_shares(assets)
    }

    /// Total issued receipt tokens.
    /// AKA total shares in the vault.
    /// AKA Total circulating supply of the receipt token.
    pub fn total_receipt_token_supply(deps: Deps, _env: Env) -> StdResult<Uint128> {
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        StdResult::Ok(receipt_token_supply)
    }

    /// Total Staking Tokens in this vault. Including assets through staking and donations.
    pub fn total_assets(deps: Deps, env: Env) -> StdResult<Uint128> {
        UnderlyingToken::query_balance(&deps, &env)
    }

    /// Get the queued withdrawal info in this vault.
    pub fn queued_withdrawal(deps: Deps, controller: Addr) -> StdResult<QueuedWithdrawalInfo> {
        shares::get_queued_withdrawal_info(deps.storage, &controller)
    }

    /// Returns the vault information
    pub fn vault_info(deps: Deps, env: Env) -> StdResult<VaultInfoResponse> {
        let balance = UnderlyingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let underlying_token = UnderlyingToken::get_denom(deps.storage)?;
        let version = cw2::get_contract_version(deps.storage)?;
        Ok(VaultInfoResponse {
            total_shares: receipt_token_supply,
            total_assets: balance,
            router: bvs_vault_base::router::get_router(deps.storage)?,
            pauser: bvs_pauser::api::get_pauser(deps.storage)?,
            operator: bvs_vault_base::router::get_operator(deps.storage)?,
            asset_id: format!(
                "cosmos:{}/bank:{}",
                env.block.chain_id,
                underlying_token.as_str()
            ),
            asset_type: AssetType::Bank,
            asset_reference: underlying_token,
            contract: version.contract,
            version: version.version,
        })
    }
}

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 2.0.0 (new)
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
