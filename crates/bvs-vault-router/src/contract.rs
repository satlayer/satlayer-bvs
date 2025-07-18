#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{set_registry, GUARDRAIL};
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_STRING_BYTES: usize = 250;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    let registry = deps.api.addr_validate(&msg.registry)?;
    set_registry(deps.storage, &registry)?;

    let pauser = deps.api.addr_validate(&msg.pauser)?;
    bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

    let guardrail = deps.api.addr_validate(&msg.guardrail)?;
    GUARDRAIL.save(deps.storage, &guardrail)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner)
        .add_attribute("pauser", pauser))
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
        ExecuteMsg::SetVault { vault, whitelisted } => {
            let vault = deps.api.addr_validate(&vault)?;
            execute::set_vault(deps, env, info, vault, whitelisted)
        }
        ExecuteMsg::SetWithdrawalLockPeriod {
            0: withdrawal_lock_period,
        } => execute::set_withdrawal_lock_period(deps, env, info, withdrawal_lock_period),
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(ContractError::Ownership)
        }
        ExecuteMsg::RequestSlashing(payload) => execute::request_slashing(deps, env, info, payload),
        ExecuteMsg::LockSlashing(id) => execute::lock_slashing(deps, env, info, id),
        ExecuteMsg::CancelSlashing(slashing_request_id) => {
            execute::cancel_slashing(deps, env, info, slashing_request_id)
        }
        ExecuteMsg::FinalizeSlashing(id) => execute::finalize_slashing(deps, info, id),
    }
}

/// This can only be called by the contract ADMIN, enforced by `wasmd` separate from cosmwasm.
/// See https://github.com/CosmWasm/cosmwasm/issues/926#issuecomment-851259818
///
/// #### 1.0.0 to 2.0.0
/// New `OPERATOR_VAULTS: Map<(&Addr, &Addr), ()>` is created to allow vaults to be queried by
/// operator.
/// The existing `VAULTS` iterated over and added to `OPERATOR_VAULTS`.
/// The `GUARDRAIL` contract is added to the router.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(mut deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let old_version =
        cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match old_version.major {
        1 => {
            migrate::vaults_to_index_operator(deps.branch())?;
            migrate::add_guardrail_to_state(deps, msg)?;
            Ok(Response::default())
        }
        _ => Ok(Response::default()),
    }
}

mod migrate {
    use super::*;
    use crate::state::{OPERATOR_VAULTS, VAULTS};

    pub fn vaults_to_index_operator(deps: DepsMut) -> Result<Response, ContractError> {
        let vaults = VAULTS
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        for vault in vaults {
            let vault_info = vault::get_vault_info(deps.as_ref(), &vault)?;

            OPERATOR_VAULTS.save(deps.storage, (&vault_info.operator, &vault), &())?;
        }

        Ok(Response::default())
    }

    pub fn add_guardrail_to_state(
        deps: DepsMut,
        msg: MigrateMsg,
    ) -> Result<Response, ContractError> {
        let guardrail = deps.api.addr_validate(&msg.guardrail)?;
        GUARDRAIL.save(deps.storage, &guardrail)?;

        Ok(Response::default())
    }
}

mod execute {
    use super::*;
    use crate::contract::query::get_withdrawal_lock_period;
    use crate::contract::vault::AssetType;
    use crate::error::ContractError;
    use crate::msg::{RequestSlashingPayload, RequestSlashingResponse};
    use crate::state::{
        self, SlashingRequest, SlashingRequestStatus, DEFAULT_WITHDRAWAL_LOCK_PERIOD,
        SLASHING_REQUESTS, SLASHING_REQUEST_EXPIRY_WINDOW, SLASH_LOCKED, WITHDRAWAL_LOCK_PERIOD,
    };
    use crate::ContractError::{InvalidSlashingRequest, Unauthorized};
    use bvs_library::addr::Operator;
    use bvs_library::ownership;
    use bvs_library::slashing::SlashingRequestId;
    use bvs_registry::msg::{
        IsOperatorOptedInToSlashingResponse, SlashingParametersResponse, StatusResponse,
    };
    use bvs_registry::RegistrationStatus;
    use cosmwasm_std::{
        Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, Env, Event, MessageInfo, Response, Uint128,
        Uint64,
    };
    use cw3::ProposalResponse;

    /// Set the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    /// After `whitelisting` a vault, the router allows the vault to accept deposits.
    /// See [`query::is_whitelisted`] for more information.
    pub fn set_vault(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        vault: Addr,
        whitelisted: bool,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        // Only for whitelisted vault:
        // - we assert that the vault is connected to the router.
        // - we save the operator to vault mapping
        //   if a vault is never whitelisted, it won't be added to the operator mapping.
        // Otherwise for non-whitelisted, the `state::VAULTS` will only be updated.
        // This is to allow the vault to be effectively removed
        // by setting `whitelisted: false` without checks from the router in
        // case the vault is malformed or broken.
        if whitelisted {
            let vault_info = vault::get_vault_info(deps.as_ref(), &vault)?;

            // The vault is not connected to this router.
            if vault_info.router != env.contract.address {
                return Err(ContractError::VaultError {
                    msg: "Vault must be connected to this router".to_string(),
                });
            }

            state::OPERATOR_VAULTS.save(deps.storage, (&vault_info.operator, &vault), &())?;
        }

        state::VAULTS.save(deps.storage, &vault, &state::Vault { whitelisted })?;

        Ok(Response::new().add_event(
            Event::new("VaultUpdated")
                .add_attribute("vault", vault)
                .add_attribute("whitelisted", whitelisted.to_string()),
        ))
    }

    pub fn set_withdrawal_lock_period(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        withdrawal_lock_period: Uint64,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        if withdrawal_lock_period.is_zero() {
            return Err(ContractError::VaultError {
                msg: "Withdrawal lock period must be greater than zero".to_string(),
            });
        }

        let prev_withdrawal_lock_period = WITHDRAWAL_LOCK_PERIOD
            .may_load(deps.storage)?
            .unwrap_or(DEFAULT_WITHDRAWAL_LOCK_PERIOD);

        WITHDRAWAL_LOCK_PERIOD.save(deps.storage, &withdrawal_lock_period)?;

        Ok(Response::new().add_event(
            Event::new("SetWithdrawalLockPeriod")
                .add_attribute(
                    "prev_withdrawal_lock_period",
                    prev_withdrawal_lock_period.to_string(),
                )
                .add_attribute(
                    "new_withdrawal_lock_period",
                    withdrawal_lock_period.to_string(),
                ),
        ))
    }

    pub fn request_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        data: RequestSlashingPayload,
    ) -> Result<Response, ContractError> {
        // service is the sender
        let service = info.sender;

        let operator: Operator = deps.api.addr_validate(data.operator.as_str())?;

        let registry = state::get_registry(deps.storage)?;

        // ensure that metadata.reason does not exceed MAX_STRING_BYTES bytes
        if data.metadata.reason.len() > MAX_STRING_BYTES {
            return Err(InvalidSlashingRequest {
                msg: "Reason exceeds maximum allowed length".to_string(),
            });
        }

        // ensure that bips is greater than zero
        if data.bips < 1 {
            return Err(InvalidSlashingRequest {
                msg: "Slashing bips must be greater than zero".to_string(),
            });
        }

        // max_slashable_delay is equal to the withdrawal lock period (7 days default)
        let max_slashable_delay = get_withdrawal_lock_period(deps.as_ref())?;
        // ensure the timestamp of slash must not be more than MAX_SLASHABLE_DELAY before and not in the future
        if data.timestamp < env.block.time.minus_seconds(max_slashable_delay.u64())
            || data.timestamp > env.block.time
        {
            return Err(InvalidSlashingRequest {
                msg: "Slash timestamp must be within the allowable slash period".to_string(),
            });
        }

        // require active status between operator and service
        let StatusResponse(operator_service_status) = deps.querier.query_wasm_smart(
            registry.to_string(),
            &bvs_registry::msg::QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: Some(data.timestamp.seconds()),
            },
        )?;

        // this check validate the service calling the locking is the same as the one that
        // requested the slashing
        if operator_service_status != u8::from(RegistrationStatus::Active) {
            return Err(InvalidSlashingRequest {
                msg: "Service and Operator must be active at the specified timestamp".to_string(),
            });
        }

        // get slashing params of the service at the given timestamp, also checks if slashing is enabled
        let SlashingParametersResponse(slashing_parameters) = deps.querier.query_wasm_smart(
            registry.clone(),
            &bvs_registry::msg::QueryMsg::SlashingParameters {
                service: service.to_string(),
                timestamp: Some(data.timestamp.seconds()),
            },
        )?;
        let slashing_parameters = match slashing_parameters {
            Some(x) => x,
            None => {
                return Err(InvalidSlashingRequest {
                    msg: "Service must have slashing enabled at the specified timestamp"
                        .to_string(),
                })
            }
        };

        // ensure bips must not exceed max_slashing_bips set by service
        if data.bips > slashing_parameters.max_slashing_bips {
            return Err(InvalidSlashingRequest {
                msg: "Slashing bips exceeds the maximum allowed by service".to_string(),
            });
        }

        // ensure that the operator has opted in to slashing at the given timestamp.
        let IsOperatorOptedInToSlashingResponse(is_operator_opted_in) =
            deps.querier.query_wasm_smart(
                registry,
                &bvs_registry::msg::QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: Some(data.timestamp.seconds()),
                },
            )?;

        if !is_operator_opted_in {
            return Err(InvalidSlashingRequest {
                msg: "Operator must be opted-in to slashing at the specified timestamp".to_string(),
            });
        }

        // get current pending slashing request for (service, operator) pair
        let prev_slashing_request =
            state::get_pending_slashing_request(deps.storage, &service, &operator)?;

        // refer to the [get_pending_slashing_request()] function doc for Some() or None return cases
        if let Some(prev_slashing_request) = prev_slashing_request {
            match SlashingRequestStatus::try_from(prev_slashing_request.status)? {
                SlashingRequestStatus::Pending => {
                    // slashing is pending within the expiry date
                    if prev_slashing_request.request_expiry > env.block.time {
                        return Err(ContractError::InvalidSlashingRequest {
                            msg: "Cannot process new request while previous slashing request is pending".to_string(),
                        });
                    } else {
                        // In this case, new request is eligible and
                        // will override the previous slashing request.
                        // Put the previous slashing request to canceled state.
                        let prev_slash_id = state::SLASHING_REQUEST_IDS
                            .load(deps.storage, (&service, &operator))?;
                        state::update_slashing_request_status(
                            deps.storage,
                            &prev_slash_id,
                            SlashingRequestStatus::Canceled,
                        )?;
                    }
                }
                SlashingRequestStatus::Locked => {
                    // If a slash is locked that mean previous slash is
                    // in the middle of slashing lifecycle
                    // new request are rejected
                    return Err(ContractError::InvalidSlashingRequest {
                        msg: "Cannot process new request while previous slashing request is in progress".to_string(),
                    });
                }
                SlashingRequestStatus::Canceled => {
                    // Previous slash has been canceled
                    // Cancellation can happen when:
                    // 1. Operator has refute the request
                    // 2. Slash Locking handler catches the slash is expired and cancel implicitly
                    // Eligible for new request
                }
                SlashingRequestStatus::Finalized => {
                    // Previous slash has been finalized, eligible for new request
                }
            }
        }

        let request_resolution = env
            .block
            .time
            .plus_seconds(slashing_parameters.resolution_window);

        // request_expiry will be using `resolution_window`
        // value from the timestamp's slashing_parameters,
        // instead of the most recent slashing param.
        // This ensures that both parties agree upon all parameters used.
        let request_expiry = env
            .block
            .time
            .plus_seconds(slashing_parameters.resolution_window)
            .plus_seconds(SLASHING_REQUEST_EXPIRY_WINDOW.u64());

        let new_request = SlashingRequest {
            request: data.clone(),
            request_time: env.block.time,
            request_resolution,
            request_expiry,
            status: SlashingRequestStatus::Pending.into(),
            service: service.clone(),
        };

        // save slash data
        let slashing_request_id =
            state::save_slashing_request(deps.storage, &service, &operator, &new_request)?;

        Ok(Response::new()
            .set_data(RequestSlashingResponse(slashing_request_id.clone()))
            .add_event(
                Event::new("RequestSlashing")
                    .add_attribute("service", service)
                    .add_attribute("operator", operator)
                    .add_attribute("slashing_request_id", slashing_request_id.to_string())
                    .add_attribute("reason", data.metadata.reason),
            ))
    }

    pub fn lock_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        id: SlashingRequestId,
    ) -> Result<Response, ContractError> {
        let slash_req = match state::SLASHING_REQUESTS.may_load(deps.storage, &id)? {
            Some(slash_req) => slash_req,
            None => {
                return Err(ContractError::InvalidSlashingRequest {
                    msg: "No slashing request found with the provided ID".to_string(),
                })
            }
        };

        let accused_operator = deps
            .api
            .addr_validate(slash_req.request.operator.as_str())?;

        match SlashingRequestStatus::try_from(slash_req.status)? {
            SlashingRequestStatus::Pending => {}
            SlashingRequestStatus::Locked => {
                return Err(ContractError::InvalidSlashingRequest {
                    msg: "Cannot lock a slashing request that is already locked".to_string(),
                })
            }
            SlashingRequestStatus::Canceled => {
                return Err(ContractError::InvalidSlashingRequest {
                    msg: "Cannot process a slashing request that has been cancelled".to_string(),
                })
            }
            SlashingRequestStatus::Finalized => {
                return Err(ContractError::InvalidSlashingRequest {
                    msg: "Cannot process a slashing request that has been finalized".to_string(),
                })
            }
        }

        // Check if the id is the same as the one in the request
        if info.sender != slash_req.service {
            return Err(ContractError::Unauthorized {
                msg: "Only the service that initiated the slashing request can lock it".to_string(),
            });
        }

        let now = env.block.time;

        if now > slash_req.request_expiry {
            return Err(ContractError::InvalidSlashingRequest {
                msg: "Cannot process a slashing request that has expired".to_string(),
            });
        };

        if now < slash_req.request_resolution {
            return Err(ContractError::InvalidSlashingRequest {
                msg: "Slashing request can only be locked after resolution time has elapsed"
                    .to_string(),
            });
        };

        let vaults_managed = state::OPERATOR_VAULTS
            .prefix(&accused_operator)
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        // Array instead of map to avoid the overheads
        let mut messages = Vec::new();
        for vault in vaults_managed {
            let vault_info = vault::get_vault_info(deps.as_ref(), &vault)?;
            let total_assets = vault_info.total_assets;

            // Due to the nature of the integer division involved, the result is always floored.
            let slash_absolute =
                total_assets.multiply_ratio(slash_req.request.bips, Uint128::from(10000_u128));

            SLASH_LOCKED.save(deps.storage, (id.clone(), &vault), &slash_absolute)?;

            // Can't slash lock a zero asset vault the vault will reject.
            // But we still need to populate the `SLASH_LOCKED` map to indicate
            // slash locking for given id is already done.
            if slash_absolute.is_zero() {
                continue;
            };

            let amount = vault::Amount(slash_absolute);
            let msg = vault::ExecuteMsg::SlashLocked(amount);
            let exec_msg = cosmwasm_std::WasmMsg::Execute {
                contract_addr: vault.to_string(),
                msg: to_json_binary(&msg)?,
                funds: vec![],
            };

            messages.push(exec_msg);
        }

        state::SLASHING_REQUESTS.save(
            deps.storage,
            &id,
            &SlashingRequest {
                service: slash_req.service,
                request: slash_req.request.clone(),
                request_time: slash_req.request_time,
                request_resolution: slash_req.request_resolution,
                request_expiry: slash_req.request_expiry,
                status: SlashingRequestStatus::Locked.into(),
            },
        )?;

        Ok(Response::new()
            .add_event(
                Event::new("LockSlashing")
                    .add_attribute("service", info.sender)
                    .add_attribute("operator", accused_operator)
                    .add_attribute("slashing_request_id", id.to_string())
                    .add_attribute("bips", slash_req.request.bips.to_string())
                    .add_attribute("affected_vaults", messages.len().to_string()),
            )
            .add_messages(messages))
    }

    /// Cancel a resolved slashing request that an operator has already resolved the issue.
    pub fn cancel_slashing(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        slashing_request_id: SlashingRequestId,
    ) -> Result<Response, ContractError> {
        // service should be the sender
        let service = info.sender;

        let slashing_request = SLASHING_REQUESTS
            .may_load(deps.storage, &slashing_request_id)?
            .ok_or(ContractError::InvalidSlashingRequest {
                msg: "No slashing request found with the provided ID".to_string(),
            })?;

        if slashing_request.service != service {
            return Err(ContractError::InvalidSlashingRequest {
                msg: "Only the service that initiated the slashing request can cancel it"
                    .to_string(),
            });
        }

        match slashing_request.status {
            status if status != SlashingRequestStatus::Pending => {
                return Err(ContractError::InvalidSlashingRequest {
                    msg: "Only slashing requests in pending status can be canceled".to_string(),
                });
            }
            _ => {}
        }

        let operator = deps.api.addr_validate(&slashing_request.request.operator)?;
        state::remove_slashing_request_id(deps.storage, &service, &operator)?;
        state::update_slashing_request_status(
            deps.storage,
            &slashing_request_id,
            SlashingRequestStatus::Canceled,
        )?;

        Ok(Response::new().add_event(
            Event::new("CancelSlashing")
                .add_attribute("service", service)
                .add_attribute("operator", operator)
                .add_attribute("slashing_request_id", slashing_request_id.to_string()),
        ))
    }

    pub fn finalize_slashing(
        deps: DepsMut,
        info: MessageInfo,
        id: SlashingRequestId,
    ) -> Result<Response, ContractError> {
        // Service is the sender
        let service = info.sender;

        // Check guardrail contract that slashing request is passed
        let guardrail = GUARDRAIL.load(deps.storage)?;
        let ProposalResponse {
            status: guardrail_proposal_status,
            ..
        }: ProposalResponse<Empty> = deps.querier.query_wasm_smart(
            guardrail.to_string(),
            &bvs_guardrail::msg::QueryMsg::ProposalBySlashingRequestId {
                slashing_request_id: id.clone(),
            },
        )?;
        if guardrail_proposal_status != cw3::Status::Passed {
            return Err(InvalidSlashingRequest {
                msg: "Slashing request must pass the guardrail before it can be finalized"
                    .to_string(),
            });
        }

        // Get slashing request
        let slash_req = match SLASHING_REQUESTS.may_load(deps.storage, &id)? {
            Some(slash_req) => slash_req,
            None => {
                return Err(InvalidSlashingRequest {
                    msg: "Slashing request ID does not exist".to_string(),
                })
            }
        };

        // Only service that requested slashing can finalize
        if slash_req.service != service {
            return Err(Unauthorized {
                msg: "Only the service that initiated the slashing request can finalize it"
                    .to_string(),
            });
        }

        // Only LOCKED slashing request can be finalized
        if slash_req.status != SlashingRequestStatus::Locked {
            return Err(InvalidSlashingRequest {
                msg: "Slashing request must be in locked status to be finalized".to_string(),
            });
        }

        let operator = deps
            .api
            .addr_validate(slash_req.request.operator.as_str())?;

        let registry = state::get_registry(deps.storage)?;
        // Get slashing params of the service at the given timestamp
        let SlashingParametersResponse(slashing_parameters) = deps.querier.query_wasm_smart(
            registry.clone(),
            &bvs_registry::msg::QueryMsg::SlashingParameters {
                service: service.to_string(),
                timestamp: Some(slash_req.request.timestamp.seconds()),
            },
        )?;
        let slashing_parameters = match slashing_parameters {
            Some(x) => x,
            None => {
                return Err(InvalidSlashingRequest {
                    msg: "Service must have slashing enabled at the specified timestamp"
                        .to_string(),
                })
            }
        };

        // transfer all slashed assets to the destination
        let mut transfer_msgs: Vec<CosmosMsg> = vec![];
        for locked_vault_amount in SLASH_LOCKED.prefix(id.clone()).range(
            deps.storage,
            None,
            None,
            cosmwasm_std::Order::Ascending,
        ) {
            let (affected_vault, locked_amount) = locked_vault_amount?;
            let vault_info = vault::get_vault_info(deps.as_ref(), &affected_vault)?;

            // if destination is None, leave it in the router.
            // if destination is Some, transfer to the destination.
            if slashing_parameters.destination.is_some() {
                let destination = slashing_parameters.clone().destination.unwrap().to_string();
                match vault_info.asset_type {
                    AssetType::Cw20 => {
                        // craft cw20 transfer msg
                        let msg = &cw20::Cw20ExecuteMsg::Transfer {
                            recipient: destination,
                            amount: locked_amount,
                        };
                        // convert to CosmosMsg
                        let exec_msg = cosmwasm_std::WasmMsg::Execute {
                            contract_addr: vault_info.asset_reference.to_string(),
                            msg: to_json_binary(msg)?,
                            funds: vec![],
                        };
                        transfer_msgs.push(exec_msg.into());
                    }
                    AssetType::Bank => {
                        // craft bank transfer msg
                        let exec_msg = BankMsg::Send {
                            to_address: destination,
                            amount: vec![Coin {
                                denom: vault_info.asset_reference,
                                amount: locked_amount,
                            }],
                        };
                        // convert to CosmosMsg
                        transfer_msgs.push(exec_msg.into());
                    }
                };
            }
        }

        // Remove all slash locked for the given slashing id
        state::remove_all_slash_locked_by_id(deps.storage, id.clone())?;

        // Update slash req status to `SlashingRequestStatus::Finalized`
        state::update_slashing_request_status(deps.storage, &id, SlashingRequestStatus::Finalized)?;

        // Remove (service,operator) -> slashing request id mapping
        state::remove_slashing_request_id(deps.storage, &service, &operator)?;

        Ok(Response::new()
            .add_event(
                Event::new("FinalizeSlashing")
                    .add_attribute("service", service)
                    .add_attribute("operator", operator)
                    .add_attribute("slashing_request_id", id.to_string())
                    .add_attribute(
                        "destination",
                        slashing_parameters
                            .destination
                            .unwrap_or(Addr::unchecked("".to_string())),
                    )
                    .add_attribute("affected_vaults", transfer_msgs.len().to_string()),
            )
            .add_messages(transfer_msgs))
    }
}

/// Snipped implementation of Vault's API
pub(crate) mod vault {
    use crate::error::ContractError;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Deps, Uint128};

    #[cw_serde]
    pub enum VaultInfoQueryMsg {
        VaultInfo {},
    }

    /// This is a snippet of the struct to avoid circular dependencies.
    /// This should be kept in sync with the original struct.
    /// See [`VaultInfoResponse`] for more information.
    #[cw_serde]
    pub struct VaultInfoResponse {
        /// The `vault-router` contract address
        pub router: Addr,

        /// The `operator` that this vault is delegated to
        pub operator: Addr,

        /// Total assets in the vault
        pub total_assets: Uint128,

        /// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
        pub asset_type: AssetType,

        /// The asset reference, either the token contract address or the bank denom.
        pub asset_reference: String,
    }

    /// see [`bvs_vault_base::msg`] for more information.
    #[cw_serde]
    pub enum AssetType {
        Cw20,
        Bank,
    }

    #[cw_serde]
    /// See [`bvs_vault_base::msg`] for more information.
    pub enum ExecuteMsg {
        SlashLocked(Amount),
    }

    #[cw_serde]
    pub struct Amount(pub Uint128);

    pub fn get_vault_info(deps: Deps, vault: &Addr) -> Result<VaultInfoResponse, ContractError> {
        deps.querier
            .query_wasm_smart(vault.to_string(), &VaultInfoQueryMsg::VaultInfo {})
            .map_err(|e| {
                let upstream_error = e.to_string();
                ContractError::VaultError {
                    msg: format!("Failed to query vault info for {vault}: {upstream_error}"),
                }
            })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsWhitelisted { vault } => {
            let vault = deps.api.addr_validate(&vault)?;
            to_json_binary(&query::is_whitelisted(deps, vault)?)
        }
        QueryMsg::IsValidating { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::is_validating(deps, operator)?)
        }
        QueryMsg::ListVaults { limit, start_after } => {
            let limit = limit.map_or(100, |v| v.min(100));
            let start_after = start_after
                .map(|s| deps.api.addr_validate(&s))
                .transpose()?;
            to_json_binary(&query::list_vaults(deps, limit, start_after)?)
        }
        QueryMsg::ListVaultsByOperator {
            operator,
            limit,
            start_after,
        } => {
            let limit = limit.map_or(100, |v| v.min(100));
            let start_after = start_after
                .map(|s| deps.api.addr_validate(&s))
                .transpose()?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::list_vaults_by_operator(
                deps,
                operator,
                limit,
                start_after,
            )?)
        }
        QueryMsg::WithdrawalLockPeriod {} => {
            to_json_binary(&query::get_withdrawal_lock_period(deps)?)
        }
        QueryMsg::SlashingRequestId { service, operator } => {
            let service = deps.api.addr_validate(&service)?;
            let operator = deps.api.addr_validate(&operator)?;
            to_json_binary(&query::slashing_request_id(deps, service, operator)?)
        }
        QueryMsg::SlashingRequest(id) => to_json_binary(&query::slashing_request(deps, id)?),
        QueryMsg::SlashingLocked {
            slashing_request_id,
        } => to_json_binary(&query::slash_locked(deps, slashing_request_id)?),
    }
}

mod query {
    use crate::msg::{
        SlashingLockedResponse, SlashingLockedResponseItem, SlashingRequestIdResponse,
        SlashingRequestResponse, Vault, VaultListResponse,
    };
    use crate::state::{
        self, DEFAULT_WITHDRAWAL_LOCK_PERIOD, SLASHING_REQUESTS, SLASHING_REQUEST_IDS,
    };
    use bvs_library::slashing::SlashingRequestId;
    use bvs_registry::msg::QueryMsg;
    use cosmwasm_std::{Addr, Deps, StdResult, Uint64};
    use cw_storage_plus::Bound;

    /// Returns whether the vault is whitelisted or not.
    /// Called by the vault to check if a vault can accept deposits.
    /// Only whitelisted vaults can accept deposits.
    pub fn is_whitelisted(deps: Deps, vault: Addr) -> StdResult<bool> {
        state::VAULTS
            .may_load(deps.storage, &vault)
            .map(|v| v.map(|v| v.whitelisted).unwrap_or(false))
    }

    /// Returns whether the operator is delegated or not.
    /// Called by vaults to check if they are delegated.
    /// Delegated vaults must queue withdrawals.
    pub fn is_validating(deps: Deps, operator: Addr) -> StdResult<bool> {
        let registry = state::get_registry(deps.storage)?;
        let is_operator_active: bool = deps.querier.query_wasm_smart(
            registry.to_string(),
            &QueryMsg::IsOperatorActive(operator.to_string()),
        )?;

        Ok(is_operator_active)
    }

    /// List all vaults in the router.
    /// Support pagination.
    pub fn list_vaults(
        deps: Deps,
        limit: u32,
        start_after: Option<Addr>,
    ) -> StdResult<VaultListResponse> {
        let range_max = start_after.as_ref().map(Bound::exclusive);
        let items = state::VAULTS.range(
            deps.storage,
            None,
            range_max,
            cosmwasm_std::Order::Descending,
        );

        let vaults = items
            .take(limit.try_into().unwrap())
            .map(|item| {
                let (k, v) = item?;
                Ok(Vault {
                    vault: k,
                    whitelisted: v.whitelisted,
                })
            })
            .collect::<StdResult<_>>()?;
        Ok(VaultListResponse(vaults))
    }

    pub fn get_withdrawal_lock_period(deps: Deps) -> StdResult<Uint64> {
        let value = state::WITHDRAWAL_LOCK_PERIOD
            .may_load(deps.storage)?
            .unwrap_or(DEFAULT_WITHDRAWAL_LOCK_PERIOD);

        Ok(value)
    }

    pub fn list_vaults_by_operator(
        deps: Deps,
        operator: Addr,
        limit: u32,
        start_after: Option<Addr>,
    ) -> StdResult<VaultListResponse> {
        let items = state::OPERATOR_VAULTS.prefix(&operator);

        let range_max = start_after.as_ref().map(Bound::exclusive);
        let items = items.range(
            deps.storage,
            None,
            range_max,
            cosmwasm_std::Order::Descending,
        );

        let vaults = items
            .take(limit as usize)
            .map(|item| {
                let (vault_address, _) = item?;
                let whitelisted = state::VAULTS
                    .load(deps.storage, &vault_address)?
                    .whitelisted;
                Ok(Vault {
                    vault: vault_address,
                    whitelisted,
                })
            })
            .collect::<StdResult<_>>()?;

        Ok(VaultListResponse(vaults))
    }

    pub fn slashing_request_id(
        deps: Deps,
        service: Addr,
        operator: Addr,
    ) -> StdResult<SlashingRequestIdResponse> {
        let pending_slashing_request_id =
            SLASHING_REQUEST_IDS.may_load(deps.storage, (&service, &operator))?;
        Ok(SlashingRequestIdResponse(pending_slashing_request_id))
    }

    pub fn slashing_request(
        deps: Deps,
        id: impl Into<SlashingRequestId>,
    ) -> StdResult<SlashingRequestResponse> {
        let slashing_request = SLASHING_REQUESTS.may_load(deps.storage, &id.into())?;
        Ok(SlashingRequestResponse(slashing_request))
    }

    pub fn slash_locked(deps: Deps, id: SlashingRequestId) -> StdResult<SlashingLockedResponse> {
        let locked = state::SLASH_LOCKED
            .prefix(id.clone())
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;

        Ok(SlashingLockedResponse(
            locked
                .into_iter()
                .map(|(vault, amount)| Ok(SlashingLockedResponseItem { vault, amount }))
                .collect::<StdResult<Vec<_>>>()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::vault::{AssetType, VaultInfoQueryMsg, VaultInfoResponse};
    use super::*;
    use super::{
        execute::{set_vault, set_withdrawal_lock_period},
        query::{get_withdrawal_lock_period, is_validating, is_whitelisted, list_vaults},
    };
    use crate::msg::RequestSlashingPayload;
    use crate::msg::SlashingMetadata;
    use crate::msg::{InstantiateMsg, SlashingRequestIdResponse, SlashingRequestResponse};
    use crate::state::{self, SlashingRequest, SlashingRequestStatus, SLASHING_REQUESTS};
    use crate::state::{Vault, OPERATOR_VAULTS, REGISTRY, SLASHING_REQUEST_IDS, VAULTS};
    use bvs_library::slashing::SlashingRequestId;
    use bvs_registry::msg::{IsOperatorActiveResponse, QueryMsg as RegistryQueryMsg};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, Attribute, ContractResult, Event, HexBinary, OwnedDeps, QuerierResult,
        SystemError, SystemResult, Uint128, Uint64, WasmQuery,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let pauser = deps.api.addr_make("pauser");
        let guardrail = deps.api.addr_make("guardrail");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
            pauser: pauser.to_string(),
            guardrail: guardrail.to_string(),
        };

        let response = instantiate(deps.as_mut(), env, owner_info, msg).unwrap();

        assert_eq!(
            response.attributes,
            vec![
                Attribute::new("method", "instantiate"),
                Attribute::new("owner", owner.to_string()),
                Attribute::new("pauser", pauser.to_string()),
            ]
        );
    }

    fn instantiate_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let pauser = deps.api.addr_make("pauser");
        let guardrail = deps.api.addr_make("guardrail");
        let owner_info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            registry: registry.to_string(),
            pauser: pauser.to_string(),
            guardrail: guardrail.to_string(),
        };

        instantiate(deps.as_mut(), env.clone(), owner_info.clone(), msg).unwrap();

        (deps, env, owner_info)
    }

    #[test]
    fn test_set_vault() {
        let (mut deps, env, owner_info) = instantiate_contract();

        let moved_env = env.clone();
        deps.querier
            .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                if let WasmQuery::Smart { contract_addr, msg } = req {
                    if *contract_addr == deps.api.addr_make("vault").to_string() {
                        let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                        match msg {
                            VaultInfoQueryMsg::VaultInfo {} => {
                                let response = VaultInfoResponse {
                                    router: moved_env.contract.address.clone(),
                                    operator: deps.api.addr_make("operator"),
                                    total_assets: Uint128::zero(),
                                    asset_type: AssetType::Cw20,
                                    asset_reference: deps.api.addr_make("vault").to_string(),
                                };
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.to_string(),
                        })
                    }
                } else {
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "Unsupported query".to_string(),
                    })
                }
            });

        let vault_contract_addr = deps.api.addr_make("vault");
        let operator_addr = deps.api.addr_make("operator");

        // whitelist is false
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault_contract_addr.clone(),
                false,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.attributes.len(), 0);
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("VaultUpdated")
                    .add_attribute("vault", vault_contract_addr.clone())
                    .add_attribute("whitelisted", "false")
            );

            let vault = VAULTS
                .load(deps.as_ref().storage, &vault_contract_addr)
                .unwrap();
            assert!(!vault.whitelisted);

            let is_none = OPERATOR_VAULTS
                .may_load(
                    deps.as_ref().storage,
                    (&operator_addr, &vault_contract_addr),
                )
                .unwrap()
                .is_none();
            assert!(is_none);
        }

        let vault = deps.api.addr_make("vault");
        let operator_addr = deps.api.addr_make("operator");

        // whitelist is true and set successfully
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault.clone(),
                true,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.attributes.len(), 0);
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("VaultUpdated")
                    .add_attribute("vault", vault.clone())
                    .add_attribute("whitelisted", "true")
            );

            let vault = VAULTS.load(deps.as_ref().storage, &vault).unwrap();
            assert!(vault.whitelisted);

            OPERATOR_VAULTS
                .load(
                    deps.as_ref().storage,
                    (&operator_addr, &vault_contract_addr),
                )
                .unwrap();
        }

        // whitelist is true and failed to set: No such contract
        let empty_vault = deps.api.addr_make("empty_vault");
        {
            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                empty_vault.clone(),
                true,
            );
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(
                err.to_string(),
                format!(
                    "Vault error: Failed to query vault info for {}: {} {}",
                    empty_vault,
                    "Generic error: Querier system error: No such contract:",
                    empty_vault
                )
            );
        }

        // whitelist is true and failed to set: Vault is not connected to the router
        let _new_vault = deps.api.addr_make("new_vault");
        {
            deps.querier
                .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                    if let WasmQuery::Smart { contract_addr, msg } = req {
                        if *contract_addr == deps.api.addr_make("vault").to_string() {
                            let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                            match msg {
                                VaultInfoQueryMsg::VaultInfo {} => {
                                    let response = VaultInfoResponse {
                                        router: vault_contract_addr.clone(),
                                        operator: operator_addr.clone(),
                                        total_assets: Uint128::zero(),
                                        asset_type: AssetType::Cw20,
                                        asset_reference: deps.api.addr_make("vault").to_string(),
                                    };
                                    SystemResult::Ok(ContractResult::Ok(
                                        to_json_binary(&response).unwrap(),
                                    ))
                                }
                            }
                        } else {
                            SystemResult::Err(SystemError::NoSuchContract {
                                addr: contract_addr.to_string(),
                            })
                        }
                    } else {
                        SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Unsupported query".to_string(),
                        })
                    }
                });

            let result = set_vault(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                vault.clone(),
                true,
            );
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert_eq!(
                err.to_string(),
                ContractError::VaultError {
                    msg: "Vault must be connected to this router".to_string()
                }
                .to_string()
            );
        }
    }

    #[test]
    fn test_set_and_get_withdrawal_lock_period() {
        let (mut deps, env, owner_info) = instantiate_contract();

        let withdrawal_lock_period = Uint64::new(120);

        // set withdrawal lock period successfully
        {
            let result = set_withdrawal_lock_period(
                deps.as_mut(),
                env.clone(),
                owner_info.clone(),
                withdrawal_lock_period,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("SetWithdrawalLockPeriod")
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        Uint64::new(604800).to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period.to_string()
                    )
            );
        }

        let withdrawal_lock_period1 = Uint64::new(150);

        // update withdrawal lock period successfully
        {
            let result = set_withdrawal_lock_period(
                deps.as_mut(),
                env.clone(),
                owner_info,
                withdrawal_lock_period1,
            );
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.events.len(), 1);
            assert_eq!(
                response.events[0],
                Event::new("SetWithdrawalLockPeriod")
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        withdrawal_lock_period.to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    )
            );
        }

        // query withdrawal lock period
        {
            let result = get_withdrawal_lock_period(deps.as_ref()).unwrap();
            assert_eq!(result, withdrawal_lock_period1);
        }

        // wrong permission to update withdrawal lock period successfully
        {
            let user_info = MessageInfo {
                sender: deps.api.addr_make("user"),
                funds: vec![],
            };
            let result =
                set_withdrawal_lock_period(deps.as_mut(), env, user_info, withdrawal_lock_period1);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_query_is_whitelisted() {
        let mut deps = mock_dependencies();

        let vault = deps.api.addr_make("vault");
        VAULTS
            .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
            .unwrap();

        let response = is_whitelisted(deps.as_ref(), vault).unwrap();
        assert!(response)
    }

    #[test]
    fn test_query_is_delegated() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let registry = deps.api.addr_make("registry");

        REGISTRY.save(&mut deps.storage, &registry).unwrap();

        deps.querier
            .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                if let WasmQuery::Smart { contract_addr, msg } = req {
                    if *contract_addr == deps.api.addr_make("registry").to_string() {
                        let msg: RegistryQueryMsg = from_json(msg).unwrap();
                        match msg {
                            RegistryQueryMsg::IsOperatorActive(_operator) => {
                                let response = IsOperatorActiveResponse(false);
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                            _ => SystemResult::Err(SystemError::Unknown {}),
                        }
                    } else {
                        SystemResult::Err(SystemError::NoSuchContract {
                            addr: contract_addr.to_string(),
                        })
                    }
                } else {
                    SystemResult::Err(SystemError::UnsupportedRequest {
                        kind: "Unsupported query".to_string(),
                    })
                }
            });

        let response = is_validating(deps.as_ref(), operator).unwrap();
        assert!(!response)
    }

    #[test]
    fn test_query_list_vaults() {
        let deps = mock_dependencies();

        let response = list_vaults(deps.as_ref(), 0, None).unwrap();
        assert_eq!(response.0.len(), 0)
    }

    #[test]
    fn test_query_list_vaults_by_operator() {
        let mut deps = mock_dependencies();

        let operator = deps.api.addr_make("operator");
        let mut vaults = Vec::new();

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault{i}"));
            vaults.push(vault.clone());
            VAULTS
                .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
                .unwrap();
            OPERATOR_VAULTS
                .save(&mut deps.storage, (&operator, &vault), &())
                .unwrap();
        }

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 100, None).unwrap();
        assert_eq!(response.0.len(), 10);

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 5, None).unwrap();
        assert_eq!(response.0.len(), 5);

        let mut response =
            query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 10, None).unwrap();
        assert_eq!(response.0.len(), 10);

        // check if those address came back the same
        // let's sort them first for easier comparison
        vaults.sort();
        response.0.sort_by(|a, b| a.vault.cmp(&b.vault));

        for i in 0..10 {
            assert_eq!(response.0[i].vault, vaults[i]);
        }

        // let's test pagination sync this time
        {
            let response1 =
                query::list_vaults_by_operator(deps.as_ref(), operator.clone(), 5, None).unwrap();
            assert_eq!(response1.0.len(), 5);

            let next_start_after = response1.0[4].vault.clone();

            let response2 = query::list_vaults_by_operator(
                deps.as_ref(),
                operator.clone(),
                5,
                Some(next_start_after),
            )
            .unwrap();

            assert_eq!(response2.0.len(), 5);

            let mut total_vaults = response1
                .0
                .iter()
                .chain(response2.0.iter())
                .collect::<Vec<_>>();

            total_vaults.sort_by(|a, b| a.vault.cmp(&b.vault));

            for i in 0..10 {
                assert_eq!(total_vaults[i].vault, vaults[i]);
            }
        }

        // let's have another operator with its own vaults
        let operator2 = deps.api.addr_make("operator2");

        let mut vaults2 = Vec::new();

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault2{i}"));
            vaults2.push(vault.clone());
            VAULTS
                .save(&mut deps.storage, &vault, &Vault { whitelisted: true })
                .unwrap();
            OPERATOR_VAULTS
                .save(&mut deps.storage, (&operator2, &vault), &())
                .unwrap();
        }

        let mut response =
            query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 10);

        // check if those address came back the same
        // let's sort them first for easier comparison
        vaults2.sort();
        response.0.sort_by(|a, b| a.vault.cmp(&b.vault));

        for i in 0..10 {
            assert_eq!(response.0[i].vault, vaults2[i]);
        }

        // let's test pagination sync for operator2's vaults as well
        {
            let response1 =
                query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 5, None).unwrap();
            assert_eq!(response1.0.len(), 5);

            let next_start_after = response1.0[4].vault.clone();

            let response2 = query::list_vaults_by_operator(
                deps.as_ref(),
                operator2.clone(),
                5,
                Some(next_start_after),
            )
            .unwrap();

            assert_eq!(response2.0.len(), 5);

            let mut total_vaults = response1
                .0
                .iter()
                .chain(response2.0.iter())
                .collect::<Vec<_>>();

            total_vaults.sort_by(|a, b| a.vault.cmp(&b.vault));

            for i in 0..10 {
                assert_eq!(total_vaults[i].vault, vaults2[i]);
            }
        }

        // we should have a total of 20 vaults
        let response = query::list_vaults(deps.as_ref(), 100, None).unwrap();
        assert_eq!(response.0.len(), 20);

        let non_operator = deps.api.addr_make("non_operator");

        let response =
            query::list_vaults_by_operator(deps.as_ref(), non_operator.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 0);
    }

    #[test]
    fn test_map_vault_migration() {
        let mut deps = mock_dependencies();

        let operator1 = deps.api.addr_make("operator");
        let vault1 = deps.api.addr_make("vault1");

        let operator2 = deps.api.addr_make("operator2");
        let vault2 = deps.api.addr_make("vault2");

        {
            let moved_operator1 = operator1.clone();
            let moved_vault1 = vault1.to_string();

            let moved_operator2 = operator2.clone();
            let moved_vault2 = vault2.to_string();
            deps.querier
                .update_wasm(move |req: &WasmQuery| -> QuerierResult {
                    if let WasmQuery::Smart { contract_addr, msg } = req {
                        let msg: VaultInfoQueryMsg = from_json(msg).unwrap();
                        let contract_addr = contract_addr.to_string();
                        let operator_addr = {
                            if contract_addr == moved_vault1 {
                                moved_operator1.clone()
                            } else if contract_addr == moved_vault2 {
                                moved_operator2.clone()
                            } else {
                                panic!("Unknown contract address")
                            }
                        };
                        match msg {
                            VaultInfoQueryMsg::VaultInfo {} => {
                                let response = VaultInfoResponse {
                                    router: deps.api.addr_make("router"),
                                    operator: operator_addr,
                                    total_assets: Uint128::zero(),
                                    asset_type: AssetType::Cw20,
                                    asset_reference: contract_addr,
                                };
                                SystemResult::Ok(ContractResult::Ok(
                                    to_json_binary(&response).unwrap(),
                                ))
                            }
                        }
                    } else {
                        SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: "Unsupported query".to_string(),
                        })
                    }
                });

            // operator1's vault
            VAULTS
                .save(&mut deps.storage, &vault1, &Vault { whitelisted: true })
                .unwrap();

            // operator2's vault
            VAULTS
                .save(&mut deps.storage, &vault2, &Vault { whitelisted: true })
                .unwrap();
        }

        // let's run the migration
        migrate::vaults_to_index_operator(deps.as_mut()).unwrap();

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator1.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].vault, vault1);

        let response =
            query::list_vaults_by_operator(deps.as_ref(), operator2.clone(), 100, None).unwrap();

        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].vault, vault2);
    }

    #[test]
    fn test_get_withdrawal_lock_period() {
        let deps = mock_dependencies();

        let response = get_withdrawal_lock_period(deps.as_ref()).unwrap();
        assert_eq!(response, Uint64::new(604800));
    }

    #[test]
    fn test_query_slashing_request_id() {
        let mut deps = mock_dependencies();
        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let request_id = SlashingRequestId(
            HexBinary::from_hex("dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );

        // query slashing request id before its saved => None
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator.clone()).unwrap();
        assert_eq!(res, None);

        // save to state
        let key = (&service, &operator);
        SLASHING_REQUEST_IDS
            .save(&mut deps.storage, key, &request_id)
            .expect("save slashing request id failed");

        // assert slash request id exists
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator.clone()).unwrap();
        assert_eq!(res, Some(request_id));

        // assert that slash request id for a different operator is None
        let operator2 = deps.api.addr_make("operator2");
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service.clone(), operator2).unwrap();
        assert_eq!(res, None);

        // assert that slash request id for a different service is None
        let service2 = deps.api.addr_make("service2");
        let SlashingRequestIdResponse(res) =
            query::slashing_request_id(deps.as_ref(), service2, operator).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn test_query_slashing_request() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let operator = deps.api.addr_make("operator");
        let service = deps.api.addr_make("service");
        let request_id = SlashingRequestId(
            HexBinary::from_hex("dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );
        let slash_request = SlashingRequest {
            request: RequestSlashingPayload {
                operator: operator.to_string(),
                bips: 100,
                timestamp: env.block.time,
                metadata: SlashingMetadata {
                    reason: "test".to_string(),
                },
            },
            request_time: env.block.time,
            request_resolution: env.block.time.plus_seconds(50),
            request_expiry: env.block.time.plus_seconds(100),
            status: SlashingRequestStatus::Pending.into(),
            service,
        };

        // query request_id before its saved => None
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), request_id.clone()).unwrap();
        assert_eq!(res, None);

        // save request_id
        SLASHING_REQUESTS
            .save(&mut deps.storage, &request_id, &slash_request)
            .expect("save slashing request failed");

        // assert request_id is saved
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), request_id.clone()).unwrap();
        assert_eq!(res, Some(slash_request));

        // assert that another request_id is not saved
        let random_request_id = SlashingRequestId(
            HexBinary::from_hex("0ff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );
        let SlashingRequestResponse(res) =
            query::slashing_request(deps.as_ref(), random_request_id).unwrap();
        assert_eq!(res, None);
    }

    #[test]
    fn test_query_slash_locked() {
        let mut deps = mock_dependencies();

        let request_id1 = SlashingRequestId(
            HexBinary::from_hex("dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09")
                .unwrap(),
        );
        let request_id2 = SlashingRequestId(
            HexBinary::from_hex("fff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412aaa")
                .unwrap(),
        );

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault{i}"));
            let absolute_slashed_locked = Uint128::new(1000 * (i + 1) as u128);
            state::SLASH_LOCKED
                .save(
                    &mut deps.storage,
                    (request_id1.clone(), &vault),
                    &absolute_slashed_locked,
                )
                .expect("save slashing request failed");
        }

        for i in 0..10 {
            let vault = deps.api.addr_make(&format!("vault2{i}"));
            let absolute_slashed_locked = Uint128::new(1000 * (i + 1) as u128);
            state::SLASH_LOCKED
                .save(
                    &mut deps.storage,
                    (request_id2.clone(), &vault),
                    &absolute_slashed_locked,
                )
                .expect("save slashing request failed");
        }

        let res = query::slash_locked(deps.as_ref(), request_id1.clone()).unwrap();

        assert_eq!(res.0.len(), 10);

        let res = query::slash_locked(deps.as_ref(), request_id2.clone()).unwrap();

        assert_eq!(res.0.len(), 10);
    }
}
