use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;
use crate::state::OperatorStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterOperator {
        operator: String,
        signature: SignatureWithSaltAndExpiry,
    },
    DeregisterOperator {
        operator: String,
    },
    UpdateMetadataURI {
        metadata_uri: String,
    },
    CancelSalt {
        salt: String,
    },
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Vec<u8>,
    pub salt: String,
    pub expiry: Uint64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorStatusResponse)]
    QueryOperator { operator: String },
}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorStatus,
}
