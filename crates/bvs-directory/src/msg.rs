use crate::state::OperatorBvsRegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    RegisterBvs {
        // TODO(fuxingloh): rename to contract_addr
        bvs_contract: String,
    },
    RegisterOperatorToBvs {
        operator: String,
        public_key: Binary,
        contract_addr: String,
        signature_with_salt_and_expiry: SignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromBvs {
        operator: String,
    },
    UpdateBvsMetadataUri {
        metadata_uri: String,
    },
    CancelSalt {
        salt: Binary,
    },
    SetRouting {
        delegation_manager: String,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    // expiry full-range will be under u53, it's safe to use u64 for this field.
    pub expiry: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorStatusResponse)]
    OperatorStatus { bvs: String, operator: String },

    #[returns(CalculateDigestHashResponse)]
    CalculateDigestHash {
        operator_public_key: String,
        bvs: String,
        salt: String,
        expiry: u64,
        contract_addr: String,
    },

    #[returns(IsSaltSpentResponse)]
    IsSaltSpent { operator: String, salt: String },

    #[returns(BvsInfoResponse)]
    BvsInfo { bvs_hash: String },

    #[returns(OperatorBvsRegistrationTypeHashResponse)]
    OperatorBvsRegistrationTypeHash {},

    #[returns(DomainTypeHashResponse)]
    DomainTypeHash {},

    #[returns(DomainNameResponse)]
    DomainName {},
}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorBvsRegistrationStatus,
}

#[cw_serde]
pub struct CalculateDigestHashResponse {
    pub digest_hash: Binary,
}

#[cw_serde]
pub struct IsSaltSpentResponse {
    pub is_salt_spent: bool,
}

#[cw_serde]
pub struct BvsInfoResponse {
    pub bvs_hash: String,
    pub bvs_contract: String,
}

#[cw_serde]
pub struct OperatorBvsRegistrationTypeHashResponse {
    pub operator_bvs_registration_type_hash: String,
}

#[cw_serde]
pub struct DomainTypeHashResponse {
    pub domain_type_hash: String,
}

#[cw_serde]
pub struct DomainNameResponse {
    pub domain_name: String,
}
