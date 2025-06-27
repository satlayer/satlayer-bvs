#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

const CONTRACT_NAME: &str = "crates.io:bvs-dvn-contract";
const CONTRACT_VERSION: &str = "0.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        router: deps.api.addr_validate(&msg.router)?,
        registry: deps.api.addr_validate(&msg.registry)?,
        owner: deps.api.addr_validate(&msg.owner)?,
        required_verification_threshold: msg.required_verification_threshold,
    };
    CONFIG.save(deps.storage, &config)?;

    // Register this contract as a Service in BVS Registry
    let register_as_service: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
        contract_addr: msg.registry,
        msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::RegisterAsService {
            // Metadata of the service
            metadata: bvs_registry::msg::Metadata {
                name: Some("The BVS DVN Company".to_string()),
                uri: Some("https://the-BVS-DVN-company.com".to_string()),
            },
        })?,
        funds: vec![],
    }
    .into();

    Ok(Response::new()
        .add_message(register_as_service)
        .add_attribute("method", "instantiate")
        .add_attribute("registry", config.registry)
        .add_attribute("router", config.router)
        .add_attribute("owner", config.owner)
        .add_attribute(
            "required_verification_threshold",
            config.required_verification_threshold.to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BroadcastPacket(packet) => execute::broadcast_packet(deps, env, info, packet),
        ExecuteMsg::SubmitPacket {
            packet_guid,
            payload_hash,
        } => execute::submit_packet(deps, env, info, packet_guid, payload_hash),
        ExecuteMsg::FinalizePacketVerification { packet_guid } => {
            execute::finalize_packet_verification(deps, env, info, packet_guid)
        }
        ExecuteMsg::SlashingCancel { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_cancel(deps, env, info, operator)
        }
        ExecuteMsg::SlashingLock { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_lock(deps, env, info, operator)
        }
        ExecuteMsg::SlashingFinalize { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::slashing_finalize(deps, env, info, operator)
        }
        ExecuteMsg::RegisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::register_operator(deps, env, info, operator)
        }
        ExecuteMsg::DeregisterOperator { operator } => {
            let operator = deps.api.addr_validate(&operator)?;
            execute::deregister_operator(deps, env, info, operator)
        }
        ExecuteMsg::EnableSlashing {} => execute::enable_slashing(deps, env, info),
        ExecuteMsg::DisableSlashing {} => execute::disable_slashing(deps, env, info),
    }
}

pub mod execute {
    use crate::msg::Packet;
    use crate::state::{
        PacketFinalized, PacketStorage, PacketVerificationConfig, PacketVerified, CONFIG, PACKETS,
        PACKETS_VERIFIED,
    };
    use crate::ContractError;
    use bvs_registry::msg::StatusResponse;
    use cosmwasm_std::{
        to_json_binary, Addr, CosmosMsg, DepsMut, Env, HexBinary, MessageInfo, Order, Response,
        Uint64,
    };
    use std::ops::Mul;

    /// Broadcast a new packet for operators to verify
    pub fn broadcast_packet(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        packet: Packet,
    ) -> Result<Response, ContractError> {
        if PACKETS.has(deps.storage, &packet.guid.to_vec()) {
            return Err(ContractError::PacketAlreadyExists);
        }

        let total_active_operators = 3; // TODO: This should be queried from the registry

        let config = CONFIG.load(deps.storage)?;
        // required_verifications = total_active_operators * required_verification_threshold / 10_000
        // rounded up to the nearest integer
        let required_verifications: u64 = Uint64::from(total_active_operators)
            .mul(Uint64::from(config.required_verification_threshold))
            .u64()
            .div_ceil(10_000);

        let packet_storage = PacketStorage {
            packet: packet.clone(),
            packet_broadcasted: env.block.time,
            packet_verification_finalized: None,
            packet_verification_config: PacketVerificationConfig {
                total_active_operators,
                required_verifications,
            },
        };

        PACKETS.save(deps.storage, &packet.guid.to_vec(), &packet_storage)?;

        Ok(Response::new()
            .add_attribute("method", "BroadcastPacket")
            .add_attribute("packet_guid", packet.guid.to_hex()))
    }

    /// Operator responds with the packet hash they verified.
    pub fn submit_packet(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        packet_guid: HexBinary,
        payload_hash: HexBinary,
    ) -> Result<Response, ContractError> {
        let operator = info.sender;

        if PACKETS_VERIFIED.has(deps.storage, (&packet_guid.to_vec(), &operator)) {
            return Err(ContractError::PacketVerificationExists);
        }

        // Load the packet from storage
        let packet_storage = PACKETS.load(deps.storage, &packet_guid.to_vec())?;

        // assert that operator is actively registered to the service at the time of packet broadcast
        let config = CONFIG.load(deps.storage)?;
        let is_operator_active: StatusResponse = deps.querier.query_wasm_smart(
            config.registry,
            &bvs_registry::msg::QueryMsg::Status {
                service: env.contract.address.to_string(),
                operator: operator.to_string(),
                timestamp: Some(packet_storage.packet_broadcasted.seconds()),
            },
        )?;

        if is_operator_active != StatusResponse(1) {
            return Err(ContractError::Unauthorized {
                msg: "Operator is not active at the time of packet broadcast".to_string(),
            });
        }

        let packet_verified = PacketVerified {
            payload_hash: payload_hash.clone(),
            timestamp: env.block.time,
        };

        // Save the payload hash to the storage
        PACKETS_VERIFIED.save(
            deps.storage,
            (&packet_guid.to_vec(), &operator),
            &packet_verified,
        )?;

        Ok(Response::new()
            .add_attribute("method", "VerifiedPacket")
            .add_attribute("operator", operator.to_string())
            .add_attribute("payload_hash", payload_hash.to_hex()))
    }

    /// Finalize the packet verification by checking if the required number of verifications has been reached.
    pub fn finalize_packet_verification(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        packet_guid: HexBinary,
    ) -> Result<Response, ContractError> {
        // Load the packet from storage
        let mut packet_storage = PACKETS.load(deps.storage, &packet_guid.to_vec())?;

        // Check if the packet verification is already finalized
        if packet_storage.packet_verification_finalized.is_some() {
            return Err(ContractError::PacketAlreadyFinalized);
        }

        // get all verified packets submissions for the given packet_guid
        let packets_verified_list = PACKETS_VERIFIED
            .prefix(&packet_guid.to_vec())
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<Result<Vec<_>, _>>()?;

        // Check if the required number of verifications has been reached
        if (packets_verified_list.len() as u64)
            < packet_storage
                .packet_verification_config
                .required_verifications
        {
            return Err(ContractError::InsufficientVerifications);
        }

        // Get the most payload hash from the submissions
        let mut payload_hashes_count = std::collections::HashMap::new();
        for (_, packet_verified) in packets_verified_list {
            let payload_hash = packet_verified.payload_hash;
            *payload_hashes_count.entry(payload_hash).or_insert(0) += 1;
        }
        // Find the most common payload hash - in case of a tie, it will return the last one found
        let (payload_hash_finalized, _) = payload_hashes_count
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .ok_or(ContractError::InsufficientVerifications)?;

        // Finalize the packet verification
        packet_storage.packet_verification_finalized = Some(PacketFinalized {
            payload_hash: payload_hash_finalized.clone(),
            timestamp: env.block.time,
        });
        // update the packet storage
        PACKETS.save(deps.storage, &packet_guid.to_vec(), &packet_storage)?;

        Ok(Response::new()
            .add_attribute("method", "FinalizePacketVerification")
            .add_attribute("packet_guid", packet_guid.to_hex())
            .add_attribute("payload_hash_finalized", payload_hash_finalized.to_hex()))
    }

    /// Cancel the slashing request,
    /// this could be due to the operator temporary fault that is not malicious
    /// and has been resolved.
    pub fn slashing_cancel(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {
                msg: "Only the owner can cancel slashing requests".to_string(),
            });
        }

        // TODO: implement SlashingCancel

        Ok(Response::new()
            .add_attribute("method", "SlashingCancel")
            .add_attribute("operator", operator.to_string()))
    }

    /// Advance the slashing request to the next stage.
    pub fn slashing_lock(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        // TODO : implement SlashingLock
        Ok(Response::new()
            .add_attribute("method", "SlashingLock")
            .add_attribute("operator", operator.to_string()))
    }

    /// Finalize the slashing request.
    pub fn slashing_finalize(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        // TODO: implement SlashingFinalize
        Ok(Response::new()
            .add_attribute("method", "SlashingFinalize")
            .add_attribute("operator", operator.to_string()))
    }

    /// Register an operator to the service.
    pub fn register_operator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {
                msg: "Only the owner can register operators".to_string(),
            });
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
                operator: operator.to_string(),
            })?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "RegisterOperator")
            .add_attribute("operator", operator.to_string()))
    }

    /// Deregister an operator from the service.
    pub fn deregister_operator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: Addr,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {
                msg: "Only the owner can deregister operators".to_string(),
            });
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(
                &bvs_registry::msg::ExecuteMsg::DeregisterOperatorFromService {
                    operator: operator.to_string(),
                },
            )?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "DeregisterOperator")
            .add_attribute("operator", operator.to_string()))
    }

    /// Enable slashing for the service with the given parameters.
    pub fn enable_slashing(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {
                msg: "Only the owner can enable slashing".to_string(),
            });
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::EnableSlashing {
                slashing_parameters: bvs_registry::SlashingParameters {
                    destination: Some(env.contract.address),
                    max_slashing_bips: 500,              // 5%
                    resolution_window: 2 * 24 * 60 * 60, // 2 days in seconds
                },
            })?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "EnableSlashing"))
    }

    /// Disable slashing for the service.
    pub fn disable_slashing(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if config.owner != info.sender {
            return Err(ContractError::Unauthorized {
                msg: "Only the owner can disable slashing".to_string(),
            });
        }

        let message: CosmosMsg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: config.registry.to_string(),
            msg: to_json_binary(&bvs_registry::msg::ExecuteMsg::DisableSlashing {})?,
            funds: vec![],
        }
        .into();

        Ok(Response::new()
            .add_message(message)
            .add_attribute("method", "DisableSlashing"))
    }
}

/// Allow the contract to be reply-ed when the slashing request is created.
/// This is to allow concurrent slashing requests to be handled.
/// For example,
/// you can't have multiple slashing requests for the same operator at the same time.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: cosmwasm_std::Reply) -> StdResult<Response> {
    // Not handled. To allow slashing to fail gracefully, e.g. slashing is occupied and in-progress.
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetPacket(packet_guid) => {
            let packet_storage = query::get_packet(deps, packet_guid)?;
            Ok(to_json_binary(&packet_storage)?)
        }
        QueryMsg::GetPacketSubmission {
            packet_guid,
            operator,
        } => {
            let operator = deps.api.addr_validate(&operator)?;
            let packet_verified = query::get_packet_submission(deps, packet_guid, operator)?;
            Ok(to_json_binary(&packet_verified)?)
        }
    }
}

pub mod query {
    use crate::state::PacketStorage;
    use cosmwasm_std::{Addr, Deps, HexBinary, StdResult};

    pub fn get_packet(deps: Deps, packet_guid: HexBinary) -> StdResult<PacketStorage> {
        let packet_storage = crate::state::PACKETS.load(deps.storage, &packet_guid.to_vec())?;
        Ok(packet_storage)
    }

    pub fn get_packet_submission(
        deps: Deps,
        packet_guid: HexBinary,
        operator: Addr,
    ) -> StdResult<crate::state::PacketVerified> {
        let packet_verified = crate::state::PACKETS_VERIFIED
            .load(deps.storage, (&packet_guid.to_vec(), &operator))?;
        Ok(packet_verified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{ExecuteMsg, InstantiateMsg, Packet, QueryMsg};
    use crate::state::{PacketFinalized, PacketStorage, PacketVerificationConfig, PacketVerified};
    use bvs_registry::msg::StatusResponse;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_json, ContractResult, HexBinary, OwnedDeps, SystemError, SystemResult, Uint64,
        WasmQuery,
    };
    use sha3::Digest;

    fn mock_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let router = deps.api.addr_make("router");
        let operator = deps.api.addr_make("operator");
        let info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            registry: registry.to_string(),
            router: router.to_string(),
            owner: owner.to_string(),
            required_verification_threshold: 5000, // 50%
        };

        instantiate(deps.as_mut(), env, info, msg).unwrap();

        deps
    }

    fn create_packet() -> crate::msg::Packet {
        crate::msg::Packet {
            nonce: Uint64::new(1),
            src_eid: 1,
            sender: HexBinary::from_hex("0101010101010101010101010101010101010101").unwrap(),
            dst_eid: 2,
            receiver: HexBinary::from_hex("0202020202020202020202020202020202020202").unwrap(),
            guid: HexBinary::from_hex("0303030303030303030303030303030303030303").unwrap(),
            message: HexBinary::from_hex("0303030303030303030303030303030303030303").unwrap(),
        }
    }

    fn mock_payload_hash(packet: &Packet) -> HexBinary {
        let mut hasher = sha3::Keccak256::new();
        hasher.update(packet.guid.to_vec());
        hasher.update(packet.message.as_ref());
        let verified_packet: [u8; 32] = <[u8; 32]>::from(hasher.finalize());

        HexBinary::from(verified_packet)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let owner = deps.api.addr_make("owner");
        let registry = deps.api.addr_make("registry");
        let router = deps.api.addr_make("router");
        let operator = deps.api.addr_make("operator");
        let info = message_info(&owner, &[]);

        let msg = InstantiateMsg {
            registry: registry.to_string(),
            router: router.to_string(),
            owner: owner.to_string(),
            required_verification_threshold: 5000, // 50%
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // Check config was stored correctly
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            config,
            Config {
                registry,
                router,
                owner,
                required_verification_threshold: 5000,
            }
        );
    }

    #[test]
    fn test_broadcast_packet() {
        let mut deps = mock_contract();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let info = message_info(&owner, &[]);

        let packet = create_packet();
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::BroadcastPacket(packet.clone()),
        )
        .unwrap();

        // Check packet was stored correctly
        let packet_storage = crate::state::PACKETS
            .load(&deps.storage, &packet.guid.to_vec())
            .unwrap();

        assert_eq!(
            packet_storage,
            PacketStorage {
                packet: packet.clone(),
                packet_broadcasted: env.block.time,
                packet_verification_finalized: None,
                packet_verification_config: PacketVerificationConfig {
                    total_active_operators: 3, // Hardcoded for this test
                    required_verifications: 2, // 50% of 3 operators
                },
            }
        );
    }

    #[test]
    fn test_submit_packet() {
        let mut deps = mock_contract();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let operator = deps.api.addr_make("operator");
        let owner_info = message_info(&owner, &[]);
        let operator_info = message_info(&operator, &[]);

        // First broadcast a packet
        let packet = create_packet();
        execute(
            deps.as_mut(),
            env.clone(),
            owner_info,
            ExecuteMsg::BroadcastPacket(packet.clone()),
        )
        .unwrap();

        // Mock operator packet verification logic (off-chain)
        let payload_hash = mock_payload_hash(&packet);

        {
            // mock registry status query to always return active status
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                } => {
                    let msg: bvs_registry::msg::QueryMsg = from_json(msg).unwrap();
                    match msg {
                        bvs_registry::msg::QueryMsg::Status {
                            service: _,
                            operator: _,
                            timestamp: _,
                        } => SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&StatusResponse(1)).unwrap(),
                        )),
                        _ => panic!("unexpected query"),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        // Now operator submit verification
        execute(
            deps.as_mut(),
            env.clone(),
            operator_info,
            ExecuteMsg::SubmitPacket {
                packet_guid: packet.guid.clone(),
                payload_hash: payload_hash.clone(),
            },
        )
        .unwrap();

        // Check verification was stored correctly
        let packet_verified = crate::state::PACKETS_VERIFIED
            .load(&deps.storage, (&packet.guid.to_vec(), &operator))
            .unwrap();

        assert_eq!(
            packet_verified,
            PacketVerified {
                payload_hash: payload_hash.clone(),
                timestamp: env.block.time,
            }
        );
    }

    #[test]
    fn test_finalize_packet_verification() {
        let mut deps = mock_contract();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let operator1 = deps.api.addr_make("operator1");
        let operator2 = deps.api.addr_make("operator2");
        let owner_info = message_info(&owner, &[]);

        // First broadcast a packet
        let packet = create_packet();
        execute(
            deps.as_mut(),
            env.clone(),
            owner_info.clone(),
            ExecuteMsg::BroadcastPacket(packet.clone()),
        )
        .unwrap();

        // Mock operator packet verification logic (off-chain)
        let payload_hash = mock_payload_hash(&packet);

        // Submit from operator1
        crate::state::PACKETS_VERIFIED
            .save(
                &mut deps.storage,
                (&packet.guid.to_vec(), &operator1),
                &PacketVerified {
                    payload_hash: payload_hash.clone(),
                    timestamp: env.block.time,
                },
            )
            .unwrap();

        // Submit from operator2
        crate::state::PACKETS_VERIFIED
            .save(
                &mut deps.storage,
                (&packet.guid.to_vec(), &operator2),
                &PacketVerified {
                    payload_hash: payload_hash.clone(),
                    timestamp: env.block.time,
                },
            )
            .unwrap();

        // Now finalize the verification
        let res = execute(
            deps.as_mut(),
            env.clone(),
            owner_info,
            ExecuteMsg::FinalizePacketVerification {
                packet_guid: packet.guid.clone(),
            },
        )
        .unwrap();

        // Check that the packet was finalized
        let packet_storage = crate::state::PACKETS
            .load(&deps.storage, &packet.guid.to_vec())
            .unwrap();

        assert!(packet_storage.packet_verification_finalized.is_some());
        let finalized = packet_storage.packet_verification_finalized.unwrap();
        assert_eq!(
            finalized,
            PacketFinalized {
                payload_hash: payload_hash.clone(),
                timestamp: env.block.time,
            }
        );
    }

    #[test]
    fn test_query_get_packet() {
        let mut deps = mock_contract();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let owner_info = message_info(&owner, &[]);

        // First broadcast a packet
        let packet = create_packet();
        execute(
            deps.as_mut(),
            env.clone(),
            owner_info,
            ExecuteMsg::BroadcastPacket(packet.clone()),
        )
        .unwrap();

        // Query the packet
        let bin = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetPacket(packet.guid.clone()),
        )
        .unwrap();

        let packet_storage: PacketStorage = from_json(bin).unwrap();
        assert_eq!(
            packet_storage,
            PacketStorage {
                packet: packet.clone(),
                packet_broadcasted: env.block.time,
                packet_verification_finalized: None,
                packet_verification_config: PacketVerificationConfig {
                    total_active_operators: 3, // Hardcoded for this test
                    required_verifications: 2, // 50% of 3 operators
                },
            }
        );
    }

    #[test]
    fn test_query_get_packet_submission() {
        let mut deps = mock_contract();
        let env = mock_env();
        let owner = deps.api.addr_make("owner");
        let operator = deps.api.addr_make("operator");
        let owner_info = message_info(&owner, &[]);
        let operator_info = message_info(&operator, &[]);

        // First broadcast a packet
        let packet = create_packet();
        execute(
            deps.as_mut(),
            env.clone(),
            owner_info,
            ExecuteMsg::BroadcastPacket(packet.clone()),
        )
        .unwrap();

        // Submit verification
        let payload_hash = mock_payload_hash(&packet);

        {
            // mock registry status query to always return active status
            deps.querier.update_wasm(move |query| match query {
                WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                } => {
                    let msg: bvs_registry::msg::QueryMsg = from_json(msg).unwrap();
                    match msg {
                        bvs_registry::msg::QueryMsg::Status {
                            service: _,
                            operator: _,
                            timestamp: _,
                        } => SystemResult::Ok(ContractResult::Ok(
                            to_json_binary(&StatusResponse(1)).unwrap(),
                        )),
                        _ => panic!("unexpected query"),
                    }
                }
                _ => SystemResult::Err(SystemError::Unknown {}),
            });
        }

        execute(
            deps.as_mut(),
            env.clone(),
            operator_info,
            ExecuteMsg::SubmitPacket {
                packet_guid: packet.guid.clone(),
                payload_hash: payload_hash.clone(),
            },
        )
        .unwrap();

        // Query the submission
        let bin = query(
            deps.as_ref(),
            env,
            QueryMsg::GetPacketSubmission {
                packet_guid: packet.guid.clone(),
                operator: operator.to_string(),
            },
        )
        .unwrap();

        let packet_verified: PacketVerified = from_json(bin).unwrap();
        assert_eq!(payload_hash, packet_verified.payload_hash);
    }
}
