#[cfg(feature = "testing")]
pub mod testing;

/// This module contains the implementation of the `ownership` module.
/// - `transfer_ownership` only allows the current owner to transfer ownership to a new owner.
/// - `assert_owner` checks if the current message sender is the owner.
pub mod ownership;
