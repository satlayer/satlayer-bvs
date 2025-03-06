#![cfg(not(target_arch = "wasm32"))]
// Only exposed on unit and integration testing, not compiled to Wasm.

mod account;
mod contract;

pub use account::*;
pub use contract::*;
