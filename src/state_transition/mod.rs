mod beacon_block;
mod beacon_state;
mod context;
mod error;
mod executor;
mod presets;

pub use beacon_block::*;
pub use beacon_state::*;
pub use context::*;
pub use error::*;
pub use executor::*;

pub enum Validation {
    Enabled,
    Disabled,
}

pub mod mainnet {
    pub use super::presets::mainnet::*;
}

pub mod minimal {}