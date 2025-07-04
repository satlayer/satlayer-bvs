#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::bank;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
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
        ExecuteMsg::DepositFor(msg) => {
            msg.validate(deps.api)?;
            execute::deposit_for(deps, env, info, msg)
        }
        ExecuteMsg::QueueWithdrawalTo(msg) => {
            msg.validate(deps.api)?;
            execute::queue_withdrawal_to(deps, env, info, msg)
        }
        ExecuteMsg::RedeemWithdrawalTo(msg) => {
            msg.validate(deps.api)?;
            execute::redeem_withdrawal_to(deps, env, info, msg)
        }
        ExecuteMsg::SlashLocked(msg) => {
            msg.validate(deps.api)?;
            execute::slash_locked(deps, env, info, msg)
        }
        ExecuteMsg::SetApproveProxy(msg) => {
            msg.validate(deps.api)?;
            execute::set_approve_proxy(deps, info, msg)
        }
    }
}

mod execute {
    use crate::bank;
    use crate::bank::get_denom;
    use crate::error::ContractError;
    use bvs_vault_base::error::VaultError;
    use bvs_vault_base::msg::{
        Amount, QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams,
        SetApproveProxyParams,
    };
    use bvs_vault_base::shares::QueuedWithdrawalInfo;
    use bvs_vault_base::{offset, proxy, router, shares};
    use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, StdError};

    /// Deposit an asset (`info.funds`) into the vault through native bank transfer and receive shares.
    ///
    /// Calculation of shares to receive is done by [`assets_to_shares`](offset::VirtualOffset::assets_to_shares).  
    /// The `msg.amount` must be equal to the amount of native tokens sent in `info.funds`.  
    /// The `info.funds` must only contain one denomination, which is the same as the vault's denom.  
    /// The `msg.recipient` is the address that'll receive the shares.
    pub fn deposit_for(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: RecipientAmount,
    ) -> Result<Response, ContractError> {
        router::assert_whitelisted(&deps.as_ref(), &env)?;

        // Determine and compare the assets to be deposited from `info.funds` and `msg.amount`
        let amount_deposited = {
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
                .checked_sub(amount_deposited)
                .map_err(StdError::from)?;
            let mut vault = offset::TotalShares::load(&deps.as_ref(), before_balance)?;

            let new_shares = vault.assets_to_shares(amount_deposited)?;
            // Add shares to TOTAL_SHARES
            vault.checked_add_shares(deps.storage, new_shares)?;

            (vault, new_shares)
        };

        // Add shares to msg.recipient
        shares::add_shares(deps.storage, &msg.recipient, new_shares)?;

        Ok(Response::new().add_event(
            Event::new("DepositFor")
                .add_attribute("sender", info.sender.to_string())
                .add_attribute("recipient", msg.recipient)
                .add_attribute("assets", amount_deposited.to_string())
                .add_attribute("shares", new_shares.to_string())
                .add_attribute("total_shares", vault.total_shares().to_string()),
        ))
    }

    /// Queue shares to withdraw later.
    /// The shares are burned from `info.sender` and wait lock period to redeem withdrawal.
    /// It doesn't remove the `total_shares` and only removes the user shares, so the exchange rate is not affected.
    pub fn queue_withdrawal_to(
        deps: DepsMut,
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

        // Remove shares from the owner
        shares::sub_shares(deps.storage, &msg.owner, msg.amount)?;

        let withdrawal_lock_period: u64 =
            router::get_withdrawal_lock_period(&deps.as_ref())?.into();
        let current_timestamp = env.block.time;
        let unlock_timestamp = current_timestamp.plus_seconds(withdrawal_lock_period);

        let queued_withdrawal_info = QueuedWithdrawalInfo {
            queued_shares: msg.amount,
            unlock_timestamp,
        };

        let result = shares::update_queued_withdrawal_info(
            deps.storage,
            &msg.controller,
            queued_withdrawal_info,
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

    /// Redeem all queued shares to assets for `msg.controller` and send the assets to `msg.recipient`.
    /// The `info.sender` must be the `msg.controller` or an approved proxy.
    pub fn redeem_withdrawal_to(
        deps: DepsMut,
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

        let (vault, claimed_assets) = {
            let balance = bank::query_balance(&deps.as_ref(), &env)?;
            let mut vault = offset::TotalShares::load(&deps.as_ref(), balance)?;

            let assets = vault.shares_to_assets(queued_shares)?;
            if assets.is_zero() {
                return Err(VaultError::zero("Withdraw assets cannot be zero").into());
            }

            // Remove shares from TOTAL_SHARES
            vault.checked_sub_shares(deps.storage, queued_shares)?;

            (vault, assets)
        };

        // Setup asset transfer to msg.recipient
        let send_msg = bank::bank_send(deps.storage, &msg.recipient, claimed_assets)?;

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
                    .add_attribute("total_shares", vault.total_shares().to_string()),
            )
            .add_message(send_msg))
    }

    /// Moves the assets from the vault to the `vault-router` contract.
    /// Part of the [https://build.satlayer.xyz/getting-started/slashing](Programmable Slashing) lifecycle.
    /// This function can only be called by `vault-router`, and takes an absolute `amount` of assets to be moved.
    /// The amount is calculated and enforced by the router.
    pub fn slash_locked(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Amount,
    ) -> Result<Response, ContractError> {
        router::assert_router(deps.as_ref().storage, &info)?;

        // if the code get passed above assert_router, it means the sender is the router
        // No need to load from storage.
        let router = info.sender;

        let vault_balance = bank::query_balance(&deps.as_ref(), &env)?;

        if amount.0 > vault_balance {
            return Err(VaultError::insufficient("Not enough balance").into());
        }

        let transfer_msg = bank::bank_send(deps.storage, &router, amount.0)?;

        let event = Event::new("SlashLocked")
            .add_attribute("sender", router.to_string())
            .add_attribute("amount", amount.0.to_string())
            .add_attribute("denom", bank::get_denom(deps.storage)?);

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
        QueryMsg::QueuedWithdrawal { controller: staker } => {
            let staker = deps.api.addr_validate(&staker)?;
            to_json_binary(&query::queued_withdrawal(deps, staker)?)
        }
        QueryMsg::VaultInfo {} => to_json_binary(&query::vault_info(deps, env)?),
    }
}

mod query {
    use crate::bank;
    use bvs_vault_base::msg::{AssetType, VaultInfoResponse};
    use bvs_vault_base::{
        offset,
        shares::{self, QueuedWithdrawalInfo},
    };
    use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};

    /// Get the shares of a staker.
    pub fn shares(deps: Deps, staker: Addr) -> StdResult<Uint128> {
        shares::get_shares(deps.storage, &staker)
    }

    /// Get the assets of a staker, converted from shares held by the staker.
    pub fn assets(deps: Deps, env: Env, staker: Addr) -> StdResult<Uint128> {
        let shares = shares(deps, staker)?;
        convert_to_assets(deps, env, shares)
    }

    /// Given the number of shares, convert to assets based on the current vault exchange rate.
    pub fn convert_to_assets(deps: Deps, env: Env, shares: Uint128) -> StdResult<Uint128> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::TotalShares::load(&deps, balance)?;
        vault.shares_to_assets(shares)
    }

    /// Given assets, get the resulting shares based on the current vault exchange rate.
    pub fn convert_to_shares(deps: Deps, env: Env, assets: Uint128) -> StdResult<Uint128> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::TotalShares::load(&deps, balance)?;
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

    /// Get queued withdrawal info in this vault.
    pub fn queued_withdrawal(deps: Deps, staker: Addr) -> StdResult<QueuedWithdrawalInfo> {
        shares::get_queued_withdrawal_info(deps.storage, &staker)
    }

    /// Returns the vault information.
    pub fn vault_info(deps: Deps, env: Env) -> StdResult<VaultInfoResponse> {
        let balance = bank::query_balance(&deps, &env)?;
        let vault = offset::TotalShares::load(&deps, balance)?;
        let denom = bank::get_denom(deps.storage)?;
        let version = cw2::get_contract_version(deps.storage)?;
        Ok(VaultInfoResponse {
            total_shares: vault.total_shares(),
            total_assets: vault.total_assets(),
            router: bvs_vault_base::router::get_router(deps.storage)?,
            pauser: bvs_pauser::api::get_pauser(deps.storage)?,
            operator: bvs_vault_base::router::get_operator(deps.storage)?,
            asset_id: format!("cosmos:{}/bank:{}", env.block.chain_id, denom.as_str()),
            asset_type: AssetType::Bank,
            asset_reference: denom.to_string(),
            contract: version.contract,
            version: version.version,
        })
    }
}

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 2.0.0
/// - Vault info now return AssetType and AssetReference in VaultInfoResponse
/// - New [ExecuteMsg::SlashLocked] for the slashing lifecycle.
/// - No storage or breaking changes.
///
/// #### 0.4.0
/// - Rename the ExecuteMsg to be more explicit.
/// - No storage changes.
///
/// #### 0.3.0
/// Initial deployed version.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::query::queued_withdrawal;

    use crate::bank;
    use crate::contract::{execute, instantiate};
    use crate::msg::InstantiateMsg;
    use bvs_vault_base::msg::{QueueWithdrawalToParams, RecipientAmount, RedeemWithdrawalToParams};
    use bvs_vault_base::{offset, router, shares};
    use bvs_vault_router::msg::QueryMsg as VaultRouterMsg;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{
        coin, coins, from_json, to_json_binary, BankMsg, Coin, ContractResult, CosmosMsg, Event,
        Response, SystemError, SystemResult, Uint128, Uint64, WasmQuery,
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
    fn test_deposit_for() {
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
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
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
        let response = execute::deposit_for(
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
                Event::new("DepositFor")
                    .add_attribute("sender", info.sender.to_string())
                    .add_attribute("recipient", info.sender.to_string())
                    .add_attribute("assets", "10000")
                    .add_attribute("shares", "10000")
                    .add_attribute("total_shares", "10000"),
            )
        );

        // assert total shares increased
        let total_shares = offset::get_total_shares(&deps.storage).unwrap();
        assert_eq!(total_shares, Uint128::new(10_000));

        // assert sender shares increased
        let sender_shares = shares::get_shares(&deps.storage, &sender).unwrap();
        assert_eq!(sender_shares, Uint128::new(10_000));
    }

    #[test]
    fn test_deposit_for_multiple_coins() {
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
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                    }
                    _ => SystemResult::Err(SystemError::Unknown {}),
                });
            }

            bank::set_denom(&mut deps.storage, "stone").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(10_000, "stone"));
        }

        let info = message_info(&sender, &[coin(10_000, "stone"), coin(9900, "stone")]);
        let err = execute::deposit_for(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            RecipientAmount {
                recipient: sender.clone(),
                amount: Uint128::new(10_000),
            },
        )
        .unwrap_err();

        assert_eq!(err.to_string(), "Sent more than one denomination");
    }

    #[test]
    fn test_deposit_for_different_denom() {
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
                        SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                    }
                    _ => SystemResult::Err(SystemError::Unknown {}),
                });
            }

            bank::set_denom(&mut deps.storage, "stone").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(10_000, "stone"));
        }

        let info = message_info(&sender, &coins(10_000, "rock"));
        let err = execute::deposit_for(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            RecipientAmount {
                recipient: sender.clone(),
                amount: Uint128::new(10_000),
            },
        )
        .unwrap_err();

        assert_eq!(err.to_string(), "Must send reserve token 'stone'");
    }

    #[test]
    fn test_queue_withdrawal_to() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let withdrawal_lock_period = Uint64::new(100);

        {
            let router = deps.api.addr_make("vault_router");
            router::set_router(&mut deps.storage, &router).unwrap();
            let operator = deps.api.addr_make("operator");
            router::set_operator(&mut deps.storage, &operator).unwrap();

            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                } => {
                    let msg: VaultRouterMsg = from_json(msg).unwrap();
                    match msg {
                        VaultRouterMsg::IsWhitelisted { vault: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                        }
                        VaultRouterMsg::IsValidating { operator: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&false).unwrap()))
                        }
                        VaultRouterMsg::WithdrawalLockPeriod {} => SystemResult::Ok(
                            ContractResult::Ok(to_json_binary(&withdrawal_lock_period).unwrap()),
                        ),
                        _ => panic!("unexpected query"),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });

            bank::set_denom(&mut deps.storage, "denom").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(100_000, "denom"));
        }

        // deposit to add shares
        {
            let info = message_info(&sender, &coins(100_000, "denom"));
            execute::deposit_for(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                RecipientAmount {
                    recipient: sender.clone(),
                    amount: Uint128::new(100_000),
                },
            )
            .unwrap();
        }

        let recipient = deps.api.addr_make("recipient");
        let new_unlock_timestamp = env.block.time.plus_seconds(withdrawal_lock_period.into());

        // queue withdrawal to for the first time
        {
            let sender_info = message_info(&sender, &[]);
            let response = execute::queue_withdrawal_to(
                deps.as_mut(),
                env.clone(),
                sender_info.clone(),
                QueueWithdrawalToParams {
                    controller: sender.clone(),
                    owner: sender.clone(),
                    amount: Uint128::new(10_000),
                },
            )
            .unwrap();

            assert_eq!(
                response,
                Response::new().add_event(
                    Event::new("QueueWithdrawalTo")
                        .add_attribute("sender", sender.to_string())
                        .add_attribute("owner", sender.to_string())
                        .add_attribute("controller", sender.to_string())
                        .add_attribute("queued_shares", "10000")
                        .add_attribute(
                            "new_unlock_timestamp",
                            new_unlock_timestamp.seconds().to_string()
                        )
                        .add_attribute("total_queued_shares", "10000")
                )
            );
        }

        // queue withdrawal to for the second time
        {
            let recipient = deps.api.addr_make("recipient");
            let sender_info = message_info(&sender, &[]);
            let response = execute::queue_withdrawal_to(
                deps.as_mut(),
                env.clone(),
                sender_info.clone(),
                QueueWithdrawalToParams {
                    controller: sender.clone(),
                    owner: sender.clone(),
                    amount: Uint128::new(12_000),
                },
            )
            .unwrap();

            assert_eq!(
                response,
                Response::new().add_event(
                    Event::new("QueueWithdrawalTo")
                        .add_attribute("sender", sender.to_string())
                        .add_attribute("owner", sender.to_string())
                        .add_attribute("controller", sender.to_string())
                        .add_attribute("queued_shares", "12000")
                        .add_attribute(
                            "new_unlock_timestamp",
                            new_unlock_timestamp.seconds().to_string()
                        )
                        .add_attribute("total_queued_shares", "22000")
                )
            );
        }

        // query queued withdrawal
        {
            let result = queued_withdrawal(deps.as_ref(), sender).unwrap();
            assert_eq!(result.queued_shares, Uint128::new(22000));
            assert_eq!(result.unlock_timestamp, new_unlock_timestamp);
        }
    }

    #[test]
    fn test_queue_withdrawal_to_not_affect_exchange_rate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let withdrawal_lock_period = Uint64::new(100);

        {
            let router = deps.api.addr_make("vault_router");
            router::set_router(&mut deps.storage, &router).unwrap();
            let operator = deps.api.addr_make("operator");
            router::set_operator(&mut deps.storage, &operator).unwrap();
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                } => {
                    let msg: VaultRouterMsg = from_json(msg).unwrap();
                    match msg {
                        VaultRouterMsg::IsWhitelisted { vault: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                        }
                        VaultRouterMsg::IsValidating { operator: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                        }
                        VaultRouterMsg::WithdrawalLockPeriod {} => SystemResult::Ok(
                            ContractResult::Ok(to_json_binary(&withdrawal_lock_period).unwrap()),
                        ),
                        _ => panic!("unexpected query"),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });

            bank::set_denom(&mut deps.storage, "denom").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(100_000, "denom"));
        }

        let amount_deposited = Uint128::new(100_000);
        // deposit to add shares
        {
            let info = message_info(&sender, &coins(100_000, "denom"));
            execute::deposit_for(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                RecipientAmount {
                    recipient: sender.clone(),
                    amount: amount_deposited,
                },
            )
            .unwrap();
        }

        let after_balance = bank::query_balance(&deps.as_ref(), &env).unwrap();
        let before_balance = after_balance.checked_sub(amount_deposited).unwrap();
        let vault = offset::TotalShares::load(&deps.as_ref(), before_balance).unwrap();

        let before_queue_shares = vault.assets_to_shares(Uint128::new(1200)).unwrap();
        let before_queue_assets = vault.shares_to_assets(Uint128::new(1200)).unwrap();

        // QueueWithdrawalTo
        {
            let sender_info = message_info(&sender, &[]);
            execute::queue_withdrawal_to(
                deps.as_mut(),
                env.clone(),
                sender_info.clone(),
                QueueWithdrawalToParams {
                    controller: sender.clone(),
                    owner: sender.clone(),
                    amount: Uint128::new(10_000),
                },
            )
            .unwrap();
        }

        let after_queue_shares = vault.assets_to_shares(Uint128::new(1200)).unwrap();
        let after_queue_assets = vault.shares_to_assets(Uint128::new(1200)).unwrap();

        assert_eq!(before_queue_shares, after_queue_shares);
        assert_eq!(before_queue_assets, after_queue_assets);
    }

    #[test]
    fn test_redeem_withdrawal_to() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let sender = deps.api.addr_make("sender");
        let withdrawal_lock_period = Uint64::new(100);

        {
            let router = deps.api.addr_make("vault_router");
            router::set_router(&mut deps.storage, &router).unwrap();
            let operator = deps.api.addr_make("operator");
            router::set_operator(&mut deps.storage, &operator).unwrap();
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                } => {
                    let msg: VaultRouterMsg = from_json(msg).unwrap();
                    match msg {
                        VaultRouterMsg::IsWhitelisted { vault: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                        }
                        VaultRouterMsg::IsValidating { operator: _ } => {
                            SystemResult::Ok(ContractResult::Ok(to_json_binary(&true).unwrap()))
                        }
                        VaultRouterMsg::WithdrawalLockPeriod {} => SystemResult::Ok(
                            ContractResult::Ok(to_json_binary(&withdrawal_lock_period).unwrap()),
                        ),
                        _ => panic!("unexpected query"),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });

            bank::set_denom(&mut deps.storage, "denom").unwrap();
            deps.querier
                .bank
                .update_balance(&env.contract.address, coins(100_000, "denom"));
        }

        // deposit to add shares
        {
            let info = message_info(&sender, &coins(100_000, "denom"));
            execute::deposit_for(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                RecipientAmount {
                    recipient: sender.clone(),
                    amount: Uint128::new(100_000),
                },
            )
            .unwrap();
        }

        let recipient = deps.api.addr_make("recipient");

        // QueueWithdrawalTo for the first time
        {
            let sender_info = message_info(&sender, &[]);
            execute::queue_withdrawal_to(
                deps.as_mut(),
                env.clone(),
                sender_info.clone(),
                QueueWithdrawalToParams {
                    controller: sender.clone(),
                    owner: sender.clone(),
                    amount: Uint128::new(10_000),
                },
            )
            .unwrap();
        }

        let mut new_env = env.clone();
        new_env.block.time = new_env.block.time.plus_seconds(120);

        let sender_info = message_info(&sender, &[]);
        let response = execute::redeem_withdrawal_to(
            deps.as_mut(),
            new_env,
            sender_info.clone(),
            RedeemWithdrawalToParams {
                controller: sender.clone(),
                recipient: recipient.clone(),
            },
        )
        .unwrap();

        assert_eq!(
            response,
            Response::new()
                .add_event(
                    Event::new("RedeemWithdrawalTo")
                        .add_attribute("sender", sender.to_string())
                        .add_attribute("controller", sender.to_string())
                        .add_attribute("recipient", recipient.to_string())
                        .add_attribute("sub_shares", "10000")
                        .add_attribute("claimed_assets", "10000")
                        .add_attribute("total_shares", "90000")
                )
                .add_message(CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: "denom".to_string(),
                        amount: Uint128::new(10_000)
                    }],
                }))
        );
    }
}
