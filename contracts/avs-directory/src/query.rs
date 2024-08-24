use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary};

#[cw_serde]
pub struct DigestHashResponse {
    pub digest_hash: Binary,
}

#[cw_serde]
pub struct SaltResponse {
    pub is_salt_spent: bool,
}

#[cw_serde]
pub struct AVSInfoResponse {
    pub avs_hash: String,
    pub avs_contract: String,
    pub state_bank: String,
    pub avs_driver: String,
}

#[cw_serde]
pub struct DelegationResponse {
    pub delegation_addr: Addr,
}

#[cw_serde]
pub struct OwnerResponse {
    pub owner_addr: Addr,
}

#[cw_serde]
pub struct RegistrationTypeHashResponse {
    pub operator_avs_registration_type_hash: String,
}

#[cw_serde]
pub struct DomainTypeHashResponse {
    pub domain_type_hash: String,
}

#[cw_serde]
pub struct DomainNameResponse {
    pub domain_name: String,
}
