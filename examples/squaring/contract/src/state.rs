use cosmwasm_std::Addr;
use cw_storage_plus::Map;

/// Key = (Input, Operator)
/// Value = Output
/// Each Operator writes their own response to the output.
pub const REQUESTS: Map<(i64, &Addr), i64> = Map::new("requests");
