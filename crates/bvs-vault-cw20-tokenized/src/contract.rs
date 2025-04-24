use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw20_base::contract::instantiate as base_instantiate;
use cw20_base::contract::{execute as execute_base, query as base_query};

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
        ExecuteMsg::Base(base_msg) => execute_base(deps, env, info, base_msg).map_err(Into::into),
        ExecuteMsg::Extended(extended_msg) => {
            // Handle the extended message here
            // For example, you can call a function to handle it
            // handle_extended_msg(deps, env, info, extended_msg)
            todo!()
        }
    }
}

mod extended_execute {
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg;
    use crate::token;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::{Recipient, RecipientAmount, VaultExecuteMsg};
    use bvs_vault_base::{
        offset, router,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Timestamp};

    pub fn handle_extended_msg(
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
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;

        let assets = msg.amount;
        let (vault, new_shares) = {
            let balance = token::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let new_shares = vault.assets_to_shares(assets)?;
            // Add shares to TOTAL_SHARES
            vault.checked_add_shares(deps.storage, new_shares)?;

            (vault, new_shares)
        };

        // CW20 Transfer of asset from info.sender to contract
        let transfer_msg = token::execute_transfer_from(
            deps.storage,
            &info.sender,
            &env.contract.address,
            msg.amount,
        )?;

        // Add shares to msg.recipient
        shares::add_shares(deps.storage, &msg.recipient, new_shares)?;

        Ok(Response::new()
            .add_event(
                Event::new("Deposit")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", msg.recipient)
                    .add_attribute("assets", assets.to_string())
                    .add_attribute("shares", new_shares.to_string())
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(transfer_msg))
    }

    /// Withdraw assets from the vault by burning shares.
    ///
    /// The shares are burned from `info.sender`.  
    /// The resulting assets are transferred to `msg.recipient`.  
    /// The `TOTAL_SHARE` in the vault is reduced.  
    pub fn withdraw_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_not_validating(&deps.as_ref())?;

        // Remove shares from the info.sender
        shares::sub_shares(deps.storage, &info.sender, msg.amount)?;

        let (vault, claim_assets) = {
            let balance = token::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::VirtualOffset::load(&deps.as_ref(), balance)?;

            let assets = vault.shares_to_assets(msg.amount)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero.").into());
            }

            // Remove shares from TOTAL_SHARES
            vault.checked_sub_shares(deps.storage, msg.amount)?;

            (vault, assets)
        };

        // CW20 transfer of asset to msg.recipient
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

    /// Queue shares to withdraw later.
    /// The shares are burned from `info.sender` and wait lock period to redeem withdrawal.
    /// /// It doesn't remove the `total_shares` and only removes the user shares, so the exchange rate is not affected.
    pub fn queue_withdrawal_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        // Remove shares from the info.sender
        shares::sub_shares(deps.storage, &info.sender, msg.amount)?;

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
            let balance = token::query_balance(&deps.as_ref(), &env)?;
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
        let transfer_msg = token::execute_new_transfer(deps.storage, &msg.0, claimed_assets)?;

        // Remove staker's info
        shares::remove_queued_withdrawal_info(deps.storage, &info.sender);

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

        QueryMsg::Extended(extended_msg) => {
            // Dispatch extended vault queries
            // handle_extended_query(deps, env, extended_msg)
            todo!()
        }
    }
}
