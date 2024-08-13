use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64, Binary};
use crate::state::OperatorAVSRegistrationStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: Addr,
    pub delegation_manager: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAVS {
        contract_addr: String,
        state_bank: String,
        avs_driver: String,
    },
    RegisterOperatorToAVS {
        operator: Addr,
        public_key: String,
        contract_addr: Addr,
        signature_with_salt_and_expiry: ExecuteSignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromAVS {
        operator: Addr,
    },
    UpdateAVSMetadataURI {
        metadata_uri: String,
    },
    CancelSalt {
        salt: String,
    },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
pub enum QueryMsg {
    QueryOperatorStatus { avs: Addr, operator: Addr },
    CalculateDigestHash {
        operator_public_key: String,
        avs: Addr,
        salt: String,
        expiry: Uint64,
        chain_id: String,
        contract_addr: Addr,
    },
    IsSaltSpent { operator: Addr, salt: String },
    GetAVSInfo { avs_hash: String },
    GetDelegationManager {},
    GetOwner {},
    GetOperatorAVSRegistrationTypeHash {},
    GetDomainTypeHash {},
    GetDomainName {},
}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorAVSRegistrationStatus,
}

#[cw_serde]
pub struct IsOperatorRegisteredQueryMsg {
    pub operator: Addr,
}

#[cw_serde]
pub struct IsOperatorRegisteredResponse {
    pub registered: bool,
}

#[cw_serde]
pub struct ExecuteSignatureWithSaltAndExpiry {
    pub signature: String,
    pub salt: String,
    pub expiry: Uint64,
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}

#[cw_serde]
pub struct AVSRegisterParams {
    pub contract_addr: String,
    pub state_bank: String,
    pub avs_driver: String,
}