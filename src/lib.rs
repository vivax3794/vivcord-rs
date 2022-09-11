#![doc = include_str!("../README.md")]

#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#[macro_use]
mod macros;

pub mod api;
pub mod gateway;
pub mod datatypes;

pub use api::Api;

pub use gateway::Gateway;
pub use gateway::EventData;

pub use datatypes::Intents;
pub use datatypes::CreateMessageParams;
