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
    RegisterOperatorToAVS {
        operator: Addr,
        public_key: Binary,
        signature: SignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromAVS {
        operator: Addr,
    },
    UpdateAVSMetadataURI {
        metadata_uri: String,
    },
    CancelSalt {
        salt: Binary,
    },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
pub enum QueryMsg {
    QueryOperator { avs: Addr, operator: Addr },
    CalculateDigestHash {
        operator_public_key: Binary,
        avs: Addr,
        salt: Binary,
        expiry: Uint64,
        chain_id: String,
    },
    IsSaltSpent { operator: Addr, salt: String },
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
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}