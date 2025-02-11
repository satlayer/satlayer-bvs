use crate::query::{
    BVSInfoResponse, DelegationResponse, DigestHashResponse, DomainNameResponse,
    DomainTypeHashResponse, OwnerResponse, RegistrationTypeHashResponse, SaltResponse,
};
use crate::state::OperatorBVSRegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub delegation_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterBVS {
        bvs_contract: BVSContractParams,
    },
    RegisterOperatorToBVS {
        operator: String,
        public_key: String,
        contract_addr: String,
        signature_with_salt_and_expiry: ExecuteSignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromBVS {
        operator: String,
    },
    UpdateBVSMetadataURI {
        metadata_uri: String,
    },
    SetDelegationManager {
        delegation_manager: String,
    },
    CancelSalt {
        salt: String,
    },
    TransferOwnership {
        new_owner: String,
    },
    Pause {},
    Unpause {},
    SetPauser {
        new_pauser: String,
    },
    SetUnpauser {
        new_unpauser: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorStatusResponse)]
    GetOperatorStatus { bvs: String, operator: String },

    #[returns(DigestHashResponse)]
    CalculateDigestHash {
        operator_public_key: String,
        bvs: String,
        salt: String,
        expiry: u64,
        contract_addr: String,
    },

    #[returns(SaltResponse)]
    IsSaltSpent { operator: String, salt: String },

    #[returns(BVSInfoResponse)]
    GetBVSInfo { bvs_hash: String },

    #[returns(DelegationResponse)]
    GetDelegationManager {},

    #[returns(OwnerResponse)]
    GetOwner {},

    #[returns(RegistrationTypeHashResponse)]
    GetOperatorBVSRegistrationTypeHash {},

    #[returns(DomainTypeHashResponse)]
    GetDomainTypeHash {},

    #[returns(DomainNameResponse)]
    GetDomainName {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorBVSRegistrationStatus,
}

#[cw_serde]
pub struct ExecuteSignatureWithSaltAndExpiry {
    pub signature: String,
    pub salt: String,
    pub expiry: u64,
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: u64,
}

#[cw_serde]
pub struct BVSContractParams {
    pub bvs_contract: String,
    pub chain_name: String,
    pub chain_id: String,
}
