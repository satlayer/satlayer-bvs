use crate::query::{
    BvsInfoResponse, DelegationResponse, DigestHashResponse, DomainNameResponse,
    DomainTypeHashResponse, OwnerResponse, RegistrationTypeHashResponse, SaltResponse,
};
use crate::state::OperatorBvsRegistrationStatus;
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
    RegisterBvs {
        bvs_contract: String,
    },
    RegisterOperatorToBvs {
        operator: String,
        public_key: String,
        contract_addr: String,
        signature_with_salt_and_expiry: ExecuteSignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromBvs {
        operator: String,
    },
    UpdateBvsMetadataUri {
        metadata_uri: String,
    },
    SetDelegationManager {
        delegation_manager: String,
    },
    CancelSalt {
        salt: String,
    },
    TwoStepTransferOwnership {
        new_owner: String,
    },
    AcceptOwnership {},
    CancelOwnershipTransfer {},
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

    #[returns(BvsInfoResponse)]
    GetBvsInfo { bvs_hash: String },

    #[returns(DelegationResponse)]
    GetDelegationManager {},

    #[returns(OwnerResponse)]
    GetOwner {},

    #[returns(RegistrationTypeHashResponse)]
    GetOperatorBvsRegistrationTypeHash {},

    #[returns(DomainTypeHashResponse)]
    GetDomainTypeHash {},

    #[returns(DomainNameResponse)]
    GetDomainName {},
}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorBvsRegistrationStatus,
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
