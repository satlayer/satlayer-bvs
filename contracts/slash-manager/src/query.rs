use crate::utils::SlashDetails;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct SlashDetailsResponse {
    pub slash_details: SlashDetails,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub is_validator: bool,
}

#[cw_serde]
pub struct MinimalSlashSignatureResponse {
    pub minimal_slash_signature: u64,
}

#[cw_serde]
pub struct CalculateSlashHashResponse {
    pub message_bytes_hex: String,
}