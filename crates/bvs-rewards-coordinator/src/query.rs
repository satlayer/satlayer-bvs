use crate::msg::DistributionRoot;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct CalculateEarnerLeafHashResponse {
    pub hash_binary: Binary,
}

#[cw_serde]
pub struct CalculateTokenLeafHashResponse {
    pub hash_binary: Binary,
}

#[cw_serde]
pub struct OperatorCommissionBipsResponse {
    pub commission_bips: u16,
}

#[cw_serde]
pub struct GetDistributionRootsLengthResponse {
    pub roots_length: u64,
}

#[cw_serde]
pub struct GetCurrentDistributionRootResponse {
    pub root: DistributionRoot,
}

#[cw_serde]
pub struct GetDistributionRootAtIndexResponse {
    pub root: DistributionRoot,
}

#[cw_serde]
pub struct GetCurrentClaimableDistributionRootResponse {
    pub root: DistributionRoot,
}

#[cw_serde]
pub struct GetRootIndexFromHashResponse {
    pub root_index: u32,
}

#[cw_serde]
pub struct MerkleizeLeavesResponse {
    pub root_hash_binary: Binary,
}

#[cw_serde]
pub struct CheckClaimResponse {
    pub check_claim: bool,
}
