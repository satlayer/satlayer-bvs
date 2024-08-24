use crate::query::{
    AVSInfoResponse, DelegationResponse, DigestHashResponse, DomainNameResponse,
    DomainTypeHashResponse, OwnerResponse, RegistrationTypeHashResponse, SaltResponse,
};
use crate::state::OperatorAVSRegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub delegation_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAVS {
        avs_contract: String,
        state_bank: String,
        avs_driver: String,
    },
    RegisterOperatorToAVS {
        operator: String,
        public_key: String,
        contract_addr: String,
        signature_with_salt_and_expiry: ExecuteSignatureWithSaltAndExpiry,
    },
    DeregisterOperatorFromAVS {
        operator: String,
    },
    UpdateAVSMetadataURI {
        metadata_uri: String,
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
    GetOperatorStatus { avs: String, operator: String },

    #[returns(DigestHashResponse)]
    CalculateDigestHash {
        operator_public_key: String,
        avs: String,
        salt: String,
        expiry: u64,
        contract_addr: String,
    },

    #[returns(SaltResponse)]
    IsSaltSpent { operator: String, salt: String },

    #[returns(AVSInfoResponse)]
    GetAVSInfo { avs_hash: String },

    #[returns(DelegationResponse)]
    GetDelegationManager {},

    #[returns(OwnerResponse)]
    GetOwner {},

    #[returns(RegistrationTypeHashResponse)]
    GetOperatorAVSRegistrationTypeHash {},

    #[returns(DomainTypeHashResponse)]
    GetDomainTypeHash {},

    #[returns(DomainNameResponse)]
    GetDomainName {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct OperatorStatusResponse {
    pub status: OperatorAVSRegistrationStatus,
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
pub struct AVSRegisterParams {
    pub avs_contract: String,
    pub state_bank: String,
    pub avs_driver: String,
}
