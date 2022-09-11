#![warn(clippy::pedantic)]

pub mod api;
pub mod gateway;
pub mod datatypes;
// pub mod client;

pub use api::Api;

pub use gateway::Gateway;
pub use gateway::EventData;

pub use datatypes::Intents;
pub use datatypes::CreateMessageParams;
