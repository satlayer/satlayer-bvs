use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20_base::contract::instantiate as base_instantiate;
use cw20_base::contract::query as base_query;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::token as PrimaryStakingToken;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let router = deps.api.addr_validate(&msg.router)?;
    bvs_vault_base::router::set_router(deps.storage, &router)?;
    let operator = deps.api.addr_validate(&msg.operator)?;
    bvs_vault_base::router::set_operator(deps.storage, &operator)?;

    let cw20_contract = deps.api.addr_validate(&msg.staking_cw20_contract)?;
    PrimaryStakingToken::instantiate(deps.storage, &cw20_contract)?;

    // Assert that the contract is able
    // to query the token info to ensure that the contract is properly set up
    PrimaryStakingToken::get_token_info(&deps.as_ref())?;

    let mut response = base_instantiate(deps, env, info, msg.receipt_cw20_instantiate_base)?;

    // merge the base response with the custom response
    response = response
        .add_attribute("method", "instantiate")
        .add_attribute("pauser", pauser)
        .add_attribute("router", router)
        .add_attribute("operator", operator)
        .add_attribute("staking_cw20_contract", cw20_contract);

    Ok(response)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Base(base_msg) => {
            receipt_cw20_execute::execute_base(deps, env, info, base_msg).map_err(Into::into)
        }
        ExecuteMsg::Extended(extended_msg) => {
            vault_execute::execute_extended(deps, env, info, extended_msg)
        }
    }
}

/// cw20 compliant messages are passed to the `cw20-base` contract.
/// Except for the `Burn` and `BurnFrom` messages.
mod receipt_cw20_execute {
    use cosmwasm_std::{Addr, StdResult, Uint128};
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

    use cw20_base::contract::execute_send;
    use cw20_base::contract::execute_transfer;
    use cw20_base::contract::execute_update_minter;

    use cw20_base::allowances::execute_decrease_allowance;
    use cw20_base::allowances::execute_increase_allowance;
    use cw20_base::allowances::execute_send_from;
    use cw20_base::allowances::execute_transfer_from;

    use cw20_base::contract::execute_update_marketing;
    use cw20_base::contract::execute_upload_logo;
    use cw20_base::state::{BALANCES as RECEIPT_TOKEN_BALANCES, TOKEN_INFO as RECEIPT_TOKEN_INFO};

    /// This mint function is almost identical to the base cw20 contract's mint function
    /// down to the variables and logic.
    /// Except that it does not require the caller to be the minter.
    /// This allow self minting authority.
    pub fn mint(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        recipient: Addr,
        amount: Uint128,
    ) -> Result<Response, cw20_base::ContractError> {
        let mut config = RECEIPT_TOKEN_INFO
            .may_load(deps.storage)?
            .ok_or(cw20_base::ContractError::Unauthorized {})?;

        // update supply and enforce cap
        config.total_supply += amount;
        if let Some(limit) = config.get_cap() {
            if config.total_supply > limit {
                return Err(cw20_base::ContractError::CannotExceedCap {});
            }
        }
        RECEIPT_TOKEN_INFO.save(deps.storage, &config)?;

        // add amount to recipient balance
        let rcpt_addr = recipient;
        RECEIPT_TOKEN_BALANCES.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", rcpt_addr)
            .add_attribute("amount", amount);
        Ok(res)
    }

    pub fn execute_base(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Cw20ExecuteMsg,
    ) -> Result<Response, cw20_base::ContractError> {
        match msg {
            cw20_base::msg::ExecuteMsg::Transfer { recipient, amount } => {
                execute_transfer(deps, env, info, recipient, amount)
            }
            cw20_base::msg::ExecuteMsg::Send {
                contract,
                amount,
                msg,
            } => execute_send(deps, env, info, contract, amount, msg),
            cw20_base::msg::ExecuteMsg::Mint { .. } => {
                // not allowed
                // for the same reason burning is not allowed
                Err(cw20_base::ContractError::Unauthorized {})
            }
            cw20_base::msg::ExecuteMsg::UpdateMinter { new_minter } => {
                execute_update_minter(deps, env, info, new_minter)
            }
            cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expires,
            } => execute_increase_allowance(deps, env, info, spender, amount, expires),
            cw20_base::msg::ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expires,
            } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
            cw20_base::msg::ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => {
                execute_transfer_from(deps, env, info, owner, recipient, amount).map_err(Into::into)
            }
            cw20_base::msg::ExecuteMsg::SendFrom {
                owner,
                contract,
                amount,
                msg,
            } => {
                execute_send_from(deps, env, info, owner, contract, amount, msg).map_err(Into::into)
            }
            cw20_base::msg::ExecuteMsg::Burn { .. } => {
                // not allowed
                // can complicate and upset/desync of
                // the VirtualOffset's total shares vs
                // total supply of the receipt token
                // the only time burning should happen
                // only at successful unstaking
                Err(cw20_base::ContractError::Unauthorized {})
            }
            cw20_base::msg::ExecuteMsg::BurnFrom { .. } => {
                // not allowed
                Err(cw20_base::ContractError::Unauthorized {})
            }
            cw20_base::msg::ExecuteMsg::UpdateMarketing {
                project,
                description,
                marketing,
            } => execute_update_marketing(deps, env, info, project, description, marketing)
                .map_err(Into::into),
            cw20_base::msg::ExecuteMsg::UploadLogo(logo) => {
                execute_upload_logo(deps, env, info, logo).map_err(Into::into)
            }
        }
    }
}

/// Addtional vault logics built on top of the base cw20 contract via extended execute msg set.
/// The extended execute msg set is practically `bvs-vault-base` crate's execute msg set.
mod vault_execute {
    use crate::error::ContractError;
    use crate::token as PrimaryStakingToken;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::{Recipient, RecipientAmount, VaultExecuteMsg};
    use bvs_vault_base::{
        offset, router,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Timestamp};
    use cw20_base::contract::execute_burn as receipt_token_burn;
    use cw20_base::contract::query_balance as query_receipt_token_balance;
    use cw20_base::state::TOKEN_INFO as RECEIPT_TOKEN_INFO;

    pub fn execute_extended(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: VaultExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            VaultExecuteMsg::DepositFor(msg) => deposit_for(deps, env, info, msg),
            VaultExecuteMsg::WithdrawTo(msg) => withdraw_to(deps, env, info, msg),
            VaultExecuteMsg::QueueWithdrawalTo(msg) => queue_withdrawal_to(deps, env, info, msg),
            VaultExecuteMsg::RedeemWithdrawalTo(msg) => redeem_withdrawal_to(deps, env, info, msg),
        }
    }

    /// This executes a transfer of assets from the `info.sender` to the vault contract.
    ///
    /// New shares are minted, based on the exchange rate, to `msg.recipient`.  
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
            let balance = PrimaryStakingToken::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let new_receipt_tokens = vault.assets_to_shares(assets)?;
            // Add shares to TOTAL_SHARES
            vault.checked_add_shares(deps.storage, new_receipt_tokens)?;

            (vault, new_receipt_tokens)
        };

        // CW20 Transfer of asset from info.sender to contract
        let transfer_msg = PrimaryStakingToken::execute_transfer_from(
            deps.storage,
            &info.sender,
            &env.contract.address,
            msg.amount,
        )?;

        // critical section
        // Issue receipt token to msg.recipient
        {
            // mint new receipt token to staker
            super::receipt_cw20_execute::mint(
                deps.branch(),
                env,
                info.clone(),
                msg.recipient.clone(),
                new_receipt_tokens,
            )?;

            // TOTAL_SHARE and TOTAL_SUPPLY should be the same
            let total_receipt_token_supply = RECEIPT_TOKEN_INFO
                .may_load(deps.storage)?
                .unwrap()
                .total_supply;

            if total_receipt_token_supply != vault.total_shares() {
                // Ideally, this should never happen
                return Err(VaultError::zero(
                    "Total shares tracked in account and total supply circulating mismatch",
                )
                .into());
            }
        }

        Ok(Response::new()
            .add_event(
                Event::new("Deposit")
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
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_not_validating(&deps.as_ref())?;

        let receipt_tokens = msg.amount;

        let (vault, claim_staking_tokens) = {
            let balance = PrimaryStakingToken::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let primary_staking_tokens = vault.shares_to_assets(receipt_tokens)?;
            if primary_staking_tokens.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            // Remove shares from TOTAL_SHARES
            vault.checked_sub_shares(deps.storage, receipt_tokens)?;

            (vault, primary_staking_tokens)
        };

        // CW20 transfer of staking asset to msg.recipient
        let transfer_msg = PrimaryStakingToken::execute_new_transfer(
            deps.storage,
            &msg.recipient,
            claim_staking_tokens,
        )?;

        // Burn the receipt token from the staker
        receipt_token_burn(deps, env.clone(), info.clone(), receipt_tokens)?;

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

    /// Queue shares to withdraw later.
    /// The shares are burned from `info.sender` and wait lock period to redeem withdrawal.
    /// It doesn't remove the `total_shares` and only removes the user shares, so the exchange rate is not affected.
    pub fn queue_withdrawal_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        // make sure staker has enough receipt tokens
        // to cover the withdrawal
        // this execute func does not carry out actual withdrawal
        // and burn of receipt token
        {
            let staker_receipt_tokens_balance =
                query_receipt_token_balance(deps.as_ref(), msg.recipient.to_string())?;
            if staker_receipt_tokens_balance.balance < msg.amount {
                return Err(VaultError::insufficient("Not enough receipt tokens").into());
            }
        }

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
        deps: DepsMut,
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

        let (vault, claimed_assets) = {
            let balance = PrimaryStakingToken::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let assets = vault.shares_to_assets(queued_shares)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            // Remove shares from TOTAL_SHARES
            vault.checked_sub_shares(deps.storage, queued_shares)?;

            (vault, assets)
        };

        // CW20 transfer of asset to msg.recipient
        let transfer_msg =
            PrimaryStakingToken::execute_new_transfer(deps.storage, &msg.0, claimed_assets)?;

        // Remove staker's info
        shares::remove_queued_withdrawal_info(deps.storage, &info.sender);

        // Burn the receipt token from the staker
        // This func internally checked sub so not having enough receipt token
        // will lock the stakes forever
        receipt_token_burn(deps, env.clone(), info.clone(), queued_shares)?;

        Ok(Response::new()
            .add_event(
                Event::new("RedeemWithdrawalTo")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.0.to_string())
                    .add_attribute("sub_shares", queued_shares.to_string())
                    .add_attribute("claimed_assets", claimed_assets.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(transfer_msg))
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<cosmwasm_std::Binary> {
    match msg {
        QueryMsg::Base(base_msg) => base_query(deps, env, base_msg),
        QueryMsg::Extended(extended_msg) => vault_query::extended_query(deps, env, extended_msg),
    }
}

mod vault_query {
    use crate::token;
    use bvs_vault_base::msg::{VaultInfoResponse, VaultQueryMsg};
    use bvs_vault_base::{
        offset,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{to_json_binary, Addr, Deps, Env, StdResult, Uint128};

    pub fn extended_query(
        deps: Deps,
        env: Env,
        msg: VaultQueryMsg,
    ) -> StdResult<cosmwasm_std::Binary> {
        match msg {
            VaultQueryMsg::Shares { staker } => {
                let staker = deps.api.addr_validate(&staker)?;
                to_json_binary(&shares(deps, staker)?)
            }
            VaultQueryMsg::Assets { staker } => {
                let staker = deps.api.addr_validate(&staker)?;
                to_json_binary(&assets(deps, env, staker)?)
            }
            VaultQueryMsg::ConvertToAssets { shares } => {
                to_json_binary(&convert_to_assets(deps, env, shares)?)
            }
            VaultQueryMsg::ConvertToShares { assets } => {
                to_json_binary(&convert_to_shares(deps, env, assets)?)
            }
            VaultQueryMsg::TotalShares {} => to_json_binary(&total_shares(deps, env)?),
            VaultQueryMsg::TotalAssets {} => to_json_binary(&total_assets(deps, env)?),
            VaultQueryMsg::QueuedWithdrawal { staker } => {
                let staker = deps.api.addr_validate(&staker)?;
                to_json_binary(&queued_withdrawal(deps, staker)?)
            }
            VaultQueryMsg::VaultInfo {} => to_json_binary(&vault_info(deps, env)?),
        }
    }

    /// Get shares of the staker
    pub fn shares(deps: Deps, staker: Addr) -> StdResult<Uint128> {
        shares::get_shares(deps.storage, &staker)
    }

    /// Get the assets of a staker, converted from shares held by staker.
    pub fn assets(deps: Deps, env: Env, staker: Addr) -> StdResult<Uint128> {
        let shares = shares(deps, staker)?;
        convert_to_assets(deps, env, shares)
    }

    /// Given the number of shares, convert to assets based on the vault exchange rate.
    pub fn convert_to_assets(deps: Deps, env: Env, shares: Uint128) -> StdResult<Uint128> {
        let balance = token::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.shares_to_assets(shares)
    }

    /// Given assets, get the resulting shares based on the vault exchange rate.
    pub fn convert_to_shares(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let balance = token::query_balance(&deps, &env)?;
        let vault = offset::VirtualOffset::load(&deps, balance)?;
        vault.assets_to_shares(assets)
    }

    /// Total issued shares in this vault.
    pub fn total_shares(deps: Deps, _env: Env) -> StdResult<Uint128> {
        offset::get_total_shares(deps.storage)
    }

    /// Total assets in this vault. Including assets through staking and donations.
    pub fn total_assets(deps: Deps, env: Env) -> StdResult<Uint128> {
        token::query_balance(&deps, &env)
    }

    /// Get the queued withdrawal info in this vault.
    pub fn queued_withdrawal(deps: Deps, staker: Addr) -> StdResult<QueuedWithdrawalInfo> {
        shares::get_queued_withdrawal_info(deps.storage, &staker)
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

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
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
