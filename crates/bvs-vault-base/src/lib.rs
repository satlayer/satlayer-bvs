pub mod error;

/// Base router (aka auth) module for the vault.
pub mod router;

/// Base messages required for all vaults.
pub mod msg;

/// Virtual shares offset module for the vault to guard against inflation attacks.
pub mod offset;

/// Accounting module for vaults that tracks staker shares.
pub mod shares;

/// Module for handling and approving controller in the vault.
pub mod controller;

pub use crate::error::VaultError;
