use cosmwasm_std::to_json_binary;
use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20_base::contract::instantiate as base_instantiate;
use cw20_base::msg::InstantiateMsg as ReceiptCw20InstantiateMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg as CombinedExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_vault_cw20::token as UnderlyingToken;

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

    let cw20_contract = deps.api.addr_validate(&msg.cw20_contract)?;
    UnderlyingToken::instantiate(deps.storage, &cw20_contract)?;

    // Assert that the contract is able
    // to query the token info to ensure that the contract is properly set up
    let staking_token_info = UnderlyingToken::get_token_info(&deps.as_ref())?;

    let receipt_token_instantiate = ReceiptCw20InstantiateMsg {
        name: format!("SatLayer {}", staking_token_info.name),
        symbol: format!("sat{}", staking_token_info.symbol),
        decimals: staking_token_info.decimals,
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
        .add_attribute("cw20_contract", cw20_contract);

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
        CombinedExecuteMsg::WithdrawTo(msg) => {
            // This is the only execute msg that is not passed to the base contract
            // because it is a custom logic for this vault contract
            msg.validate(deps.api)?;
            vault_execute::withdraw_to(deps, env, info, msg)
        }
        CombinedExecuteMsg::DepositFor(msg) => {
            // This is the only execute msg that is not passed to the base contract
            // because it is a custom logic for this vault contract
            msg.validate(deps.api)?;
            vault_execute::deposit_for(deps, env, info, msg)
        }
        CombinedExecuteMsg::QueueWithdrawalTo(msg) => {
            // This is the only execute msg that is not passed to the base contract
            // because it is a custom logic for this vault contract
            msg.validate(deps.api)?;
            vault_execute::queue_withdrawal_to(deps, env, info, msg)
        }
        CombinedExecuteMsg::RedeemWithdrawalTo(msg) => {
            // This is the only execute msg that is not passed to the base contract
            // because it is a custom logic for this vault contract
            msg.validate(deps.api)?;
            vault_execute::redeem_withdrawal_to(deps, env, info, msg)
        }
        CombinedExecuteMsg::SlashLocked(msg) => {
            // This is the only execute msg that is not passed to the base contract
            // because it is a custom logic for this vault contract
            msg.validate(deps.api)?;
            vault_execute::slash_locked(deps, env, info, msg)
        }
        _ => {
            // cw20 compliant messages are passed to the `cw20-base` contract.
            // Except for the `Burn` and `BurnFrom` messages.
            receipt_cw20_execute::execute_base(deps, env, info, msg).map_err(Into::into)
        }
    }
}

/// cw20 compliant messages are passed to the `cw20-base` contract.
/// Except for the `Mint`, `Burn` and `BurnFrom` messages.
/// The only time receipt token total supply should be changed is through staking and unstaking
/// More precisely, only through - deposit_for and withdraw_to and redeem_withdrawal_to
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
    ) -> Result<Response, cw20_base::ContractError> {
        let mut config = RECEIPT_TOKEN_INFO
            .may_load(deps.storage)?
            .ok_or(cw20_base::ContractError::Unauthorized {})?;

        // update supply and enforce cap
        config.total_supply += amount;

        RECEIPT_TOKEN_INFO.save(deps.storage, &config)?;

        RECEIPT_TOKEN_BALANCES.update(
            deps.storage,
            &recipient,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", recipient)
            .add_attribute("amount", amount);
        Ok(res)
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
                // So this means sombody is trying to call a non-supported message
                Err(cw20_base::ContractError::Std(StdError::generic_err(
                    "This message is not supported",
                )))
            }
        }
    }
}

/// Additional vault logics are built on top of the base CW20 contract via an extended execute msg set.
/// The extended execute msg set is practically `bvs-vault-base` crate's execute msg set.
mod vault_execute {
    use crate::error::ContractError;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::{Recipient, RecipientAmount};
    use bvs_vault_base::{
        offset, router,
        shares::{self, QueuedWithdrawalInfo},
    };
    use bvs_vault_cw20::token as UnderlyingToken;
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Timestamp};
    use cw20_base::contract::execute_burn as receipt_token_burn;

    /// This executes a transfer of assets from the `info.sender` to the vault contract.
    ///
    /// New receipt token are minted, based on the exchange rate, to `msg.recipient`.  
    /// The `TOTAL_SHARE` in the vault is increased.
    ///
    /// ### CW20 Variant Warning
    ///
    /// Underlying assets that are not strictly CW20 compliant may cause unexpected behavior in token balances.
    /// For example, any token with a fee-on-transfer mechanism is not supported.
    ///
    /// Therefore, we do not support non-standard CW20 tokens.
    /// Vault deployed with such tokens will be blacklisted in the vault-router.
    pub fn deposit_for(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;

        let assets = msg.amount;
        let (vault, new_receipt_tokens) = {
            let staking_token_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;
            let receipt_token_supply =
                cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;
            let vault = offset::VirtualOffset::new(receipt_token_supply, staking_token_balance)?;

            let new_receipt_tokens = vault.assets_to_shares(assets)?;

            (vault, new_receipt_tokens)
        };

        // CW20 Transfer of asset from info.sender to contract
        let transfer_msg = UnderlyingToken::execute_transfer_from(
            deps.storage,
            &info.sender,
            &env.contract.address,
            msg.amount,
        )?;

        // critical section
        // Issue receipt token to msg.recipient
        {
            // mint new receipt token to staker
            super::receipt_cw20_execute::mint_internal(
                deps.branch(),
                msg.recipient.clone(),
                new_receipt_tokens,
            )?;
        }

        Ok(Response::new()
            .add_event(
                Event::new("DepositFor")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient)
                    .add_attribute("assets", assets.to_string())
                    .add_attribute("shares", new_receipt_tokens.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(transfer_msg))
    }

    /// Withdraw assets from the vault by burning receipt token.
    /// Also total shares are reduced from the VirtualOffset module to keep accounting math synced.
    /// The resulting staked assets are now unstaked and transferred to `msg.recipient`.  
    /// The `TOTAL_SHARE` in the vault is reduced.  
    pub fn withdraw_to(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_not_validating(&deps.as_ref())?;

        let receipt_tokens = msg.amount;

        let (vault, claim_staking_tokens) = {
            let staking_token_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;
            let receipt_token_supply =
                cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;
            let vault = offset::VirtualOffset::new(receipt_token_supply, staking_token_balance)?;

            let primary_staking_tokens = vault.shares_to_assets(receipt_tokens)?;
            if primary_staking_tokens.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            (vault, primary_staking_tokens)
        };

        // CW20 transfer of staking asset to msg.recipient
        let transfer_msg = UnderlyingToken::execute_new_transfer(
            deps.storage,
            &msg.recipient,
            claim_staking_tokens,
        )?;

        // Burn the receipt token from the staker
        receipt_token_burn(deps.branch(), env.clone(), info.clone(), receipt_tokens)?;

        Ok(Response::new()
            .add_event(
                Event::new("Withdraw")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient.to_string())
                    .add_attribute("assets", claim_staking_tokens.to_string())
                    .add_attribute("shares", msg.amount.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(transfer_msg))
    }

    /// Queue receipt tokens to withdraw later.
    /// The shares are burned from `info.sender` and wait lock period to redeem withdrawal.
    /// It doesn't remove the `total_supply` and only removes the user shares, so the exchange rate is not affected.
    pub fn queue_withdrawal_to(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        // ill-liquidate the receipt token from the staker
        // by moving the asset into this vault balance.
        // We can't burn until the actual unstake (redeem withdrawal) occurs.
        // due to total supply mutation can impact the exchange rate to change prematurely.
        cw20_base::contract::execute_transfer(
            deps.branch(),
            env.clone(),
            info.clone(),
            env.contract.address.to_string(),
            msg.amount,
        )?;

        let withdrawal_lock_period: u64 =
            router::get_withdrawal_lock_period(&deps.as_ref())?.into();
        let current_timestamp = env.block.time.seconds();
        let unlock_timestamp =
            Timestamp::from_seconds(withdrawal_lock_period).plus_seconds(current_timestamp);

        let new_queued_withdrawal_info = QueuedWithdrawalInfo {
            queued_shares: msg.amount,
            unlock_timestamp,
        };

        let result = shares::update_queued_withdrawal_info(
            deps.storage,
            &msg.recipient,
            new_queued_withdrawal_info,
        )?;

        Ok(Response::new().add_event(
            Event::new("QueueWithdrawalTo")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("recipient", msg.recipient.to_string())
                .add_attribute("queued_shares", msg.amount.to_string())
                .add_attribute(
                    "new_unlock_timestamp",
                    unlock_timestamp.seconds().to_string(),
                )
                .add_attribute("total_queued_shares", result.queued_shares.to_string()),
        ))
    }

    /// Redeem all queued shares to assets for `msg.recipient`.
    /// The `info.sender` must be equal to the `msg.recipient` in [`queue_withdrawal_to`].
    pub fn redeem_withdrawal_to(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Recipient,
    ) -> Result<Response, ContractError> {
        let withdrawal_info = shares::get_queued_withdrawal_info(deps.storage, &info.sender)?;
        let queued_shares = withdrawal_info.queued_shares;
        let unlock_timestamp = withdrawal_info.unlock_timestamp;

        if queued_shares.is_zero() || unlock_timestamp.seconds() == 0 {
            return Err(VaultError::zero("No queued shares").into());
        }

        if unlock_timestamp.seconds() > env.block.time.seconds() {
            return Err(VaultError::locked("The shares are locked").into());
        }

        let claimed_assets = {
            let staking_token_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;
            let receipt_token_supply =
                cw20_base::contract::query_token_info(deps.as_ref())?.total_supply;
            let vault = offset::VirtualOffset::new(receipt_token_supply, staking_token_balance)?;

            let assets = vault.shares_to_assets(queued_shares)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            assets
        };

        // CW20 transfer of asset to msg.recipient
        let transfer_msg =
            UnderlyingToken::execute_new_transfer(deps.storage, &msg.0, claimed_assets)?;

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

        // Remove staker's info
        shares::remove_queued_withdrawal_info(deps.storage, &info.sender);

        Ok(Response::new()
            .add_event(
                Event::new("RedeemWithdrawalTo")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.0.to_string())
                    .add_attribute("sub_shares", queued_shares.to_string())
                    .add_attribute("claimed_assets", claimed_assets.to_string())
                    .add_attribute("total_shares", receipt_token_supply.to_string()),
            )
            .add_message(transfer_msg))
    }

    /// Moves the assets from the vault to the `vault-router` contract.
    /// Part of the [https://build.satlayer.xyz/architecture/slashing](Programmable Slashing) lifecycle.
    /// This function can only be called by `vault-router`, and takes an absolute `amount` of assets to be moved.
    /// The amount is calculated and enforced by the router.
    pub fn slash_locked(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: bvs_vault_base::msg::Amount,
    ) -> Result<Response, ContractError> {
        router::assert_router(deps.as_ref().storage, &info)?;

        // if the code get passed above assert_router, it means the sender is the router
        // No need to load from storage.
        let router = info.sender;

        let vault_balance = UnderlyingToken::query_balance(&deps.as_ref(), &env)?;

        if amount.0 > vault_balance {
            return Err(VaultError::insufficient("Not enough balance").into());
        }

        let transfer_msg = UnderlyingToken::execute_new_transfer(deps.storage, &router, amount.0)?;

        let event = Event::new("SlashLocked")
            .add_attribute("sender", router.to_string())
            .add_attribute("amount", amount.0.to_string())
            .add_attribute(
                "token",
                UnderlyingToken::get_cw20_contract(deps.storage)?.to_string(),
            );

        Ok(Response::new().add_event(event).add_message(transfer_msg))
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
        QueryMsg::ConvertToAssets { shares } => {
            to_json_binary(&vault_query::convert_to_staking_token(deps, env, shares)?)
        }
        QueryMsg::ConvertToShares { assets } => {
            to_json_binary(&vault_query::convert_to_receipt_token(deps, env, assets)?)
        }
        QueryMsg::TotalShares {} => {
            to_json_binary(&vault_query::total_receipt_token_supply(deps, env)?)
        }
        QueryMsg::TotalAssets {} => to_json_binary(&vault_query::total_assets(deps, env)?),
        QueryMsg::QueuedWithdrawal { staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&vault_query::queued_withdrawal(deps, staker)?)
        }
        QueryMsg::VaultInfo {} => to_json_binary(&vault_query::vault_info(deps, env)?),
        _ => {
            // cw20 compliant messages are passed to the `cw20-base` contract.
            // Except for the `Burn` and `BurnFrom` messages.
            cw20_base::contract::query(deps, env, msg.into())
        }
    }
}

mod vault_query {
    use bvs_vault_base::msg::VaultInfoResponse;
    use bvs_vault_base::{
        offset,
        shares::{self, QueuedWithdrawalInfo},
    };
    use bvs_vault_cw20::token as StakingToken;
    use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
    use cw20_base::contract::query_balance;

    /// Get shares of the staker
    /// Since this vault is tokenized, shares are practically the receipt token.
    /// Such that quering shares is equivalent to querying the receipt token balance of a
    /// particular staker/address.
    /// But we will support this query to keep the API consistent with the non-tokenized vault.
    /// Hopefully that helps with contract consumer/frontend to minimize code changes.
    pub fn balance_of(deps: Deps, staker: String) -> StdResult<Uint128> {
        // this func come from the cw20_base crate
        // validate the staker address
        let balance = query_balance(deps, staker)?;

        StdResult::Ok(balance.balance)
    }

    /// Get the staking token of a staker, converted from receipt_tokens held by staker.
    pub fn assets(deps: Deps, env: Env, staker: Addr) -> StdResult<Uint128> {
        let balance = query_balance(deps, staker.to_string())?;
        convert_to_staking_token(deps, env, balance.balance)
    }

    /// Given the number of receipt_token, convert to staking token based on the vault exchange rate.
    pub fn convert_to_staking_token(
        deps: Deps,
        env: Env,
        receipt_tokens: Uint128,
    ) -> StdResult<Uint128> {
        let staking_token_balance = StakingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let vault = offset::VirtualOffset::new(receipt_token_supply, staking_token_balance)?;
        vault.shares_to_assets(receipt_tokens)
    }

    /// Given assets, get the resulting shares based on the vault exchange rate.
    /// Shares in this tokenized vault the receipt token.
    /// Keeping the msg name the same as the non-tokenized vault for consistency.
    pub fn convert_to_receipt_token(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let staking_token_balance = StakingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let vault = offset::VirtualOffset::new(receipt_token_supply, staking_token_balance)?;
        vault.assets_to_shares(assets)
    }

    /// Total issued receipt tokens.
    /// AKA total shares in the vault.
    /// AKA Total cirulating supply of the receipt token.
    pub fn total_receipt_token_supply(deps: Deps, _env: Env) -> StdResult<Uint128> {
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        StdResult::Ok(receipt_token_supply)
    }

    /// Total Staking Tokens in this vault. Including assets through staking and donations.
    pub fn total_assets(deps: Deps, env: Env) -> StdResult<Uint128> {
        StakingToken::query_balance(&deps, &env)
    }

    /// Get the queued withdrawal info in this vault.
    pub fn queued_withdrawal(deps: Deps, staker: Addr) -> StdResult<QueuedWithdrawalInfo> {
        shares::get_queued_withdrawal_info(deps.storage, &staker)
    }

    /// Returns the vault information
    pub fn vault_info(deps: Deps, env: Env) -> StdResult<VaultInfoResponse> {
        let balance = StakingToken::query_balance(&deps, &env)?;
        let receipt_token_supply = cw20_base::contract::query_token_info(deps)?.total_supply;
        let cw20_contract = StakingToken::get_cw20_contract(deps.storage)?;
        let version = cw2::get_contract_version(deps.storage)?;
        Ok(VaultInfoResponse {
            total_shares: receipt_token_supply,
            total_assets: balance,
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
mod tests {
    use bvs_vault_base::shares::{
        add_shares, get_queued_withdrawal_info, get_shares, remove_queued_withdrawal_info,
        sub_shares, update_queued_withdrawal_info, QueuedWithdrawalInfo,
    };
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{Addr, Timestamp, Uint128};

    #[test]
    fn get_zero_shares() {
        let store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());
    }

    #[test]
    fn add_and_get_shares() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(24690));
    }

    #[test]
    fn add_and_sub_shares() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        let remove_shares = Uint128::new(1234);
        sub_shares(&mut store, &staker, remove_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(11_111));
    }

    #[test]
    fn set_and_get_queued_withdrawal_info() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");

        let result = get_queued_withdrawal_info(&store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::zero());
        assert_eq!(result.unlock_timestamp, Timestamp::from_seconds(0));

        let queued_withdrawal_info1 = QueuedWithdrawalInfo {
            queued_shares: Uint128::new(123),
            unlock_timestamp: Timestamp::from_seconds(0),
        };

        let result =
            update_queued_withdrawal_info(&mut store, &staker, queued_withdrawal_info1.clone())
                .unwrap();
        assert_eq!(result.queued_shares, queued_withdrawal_info1.queued_shares);
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info1.unlock_timestamp
        );

        let queued_withdrawal_info2 = QueuedWithdrawalInfo {
            queued_shares: Uint128::new(456),
            unlock_timestamp: Timestamp::from_seconds(0),
        };

        let result =
            update_queued_withdrawal_info(&mut store, &staker, queued_withdrawal_info2.clone())
                .unwrap();
        assert_eq!(result.queued_shares, Uint128::new(579));
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info2.unlock_timestamp
        );

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::new(579));
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info2.unlock_timestamp
        );

        remove_queued_withdrawal_info(&mut store, &staker);

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::zero());
        assert_eq!(result.unlock_timestamp, Timestamp::from_seconds(0));
    }
}
