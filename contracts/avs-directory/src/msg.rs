use cosmwasm_schema::{cw_serde, QueryResponses};
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
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorStatusResponse)]
    QueryOperator { operator: Addr },
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