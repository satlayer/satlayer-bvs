pub mod contract;
pub mod msg;
pub mod testing;

mod error;

mod state;

pub use crate::error::ContractError;
pub use crate::state::RegistrationStatus;
pub use crate::state::SlashingParameters;
