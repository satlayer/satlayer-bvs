use cw_storage_plus::Strategy;

/// EVERY_SECOND is an alias for `Strategy::EveryBlock`.
/// This is possible because Snapshot accepts u64 as parameter for block height.
/// Since Timestamp is an u64 and has similar properties (monotonic, etc.),
/// it can be substituted for block height.
pub const EVERY_SECOND: Strategy = Strategy::EveryBlock;
