use serde::{Deserialize, Serialize};

use super::Snowflake;

/// Discord Message
/// 
/// # Important
/// You should never need to construct this struct yourself,
/// This should be created by `vivcord`, usually in [`MessageCreate`][crate::EventData] events.
#[derive(Deserialize, Clone)]
pub struct Message {
    /// Message id 
    pub id: Snowflake,
    /// Id of channel where this message was sent
    pub channel_id: Snowflake,
    /// Text content of message
    pub content: String,
    // TODO: Implement all fields
}

to_snowflake_simple!(Message);
comp_by_field!(Message, self.id);

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Messages have way to many fields
        // lets only show the important stuff
        // TODO: add author field once that is added
        f.debug_struct("Message")
            .field("content", &self.content)
            .finish_non_exhaustive()
    }
}

/// Fields that can be passed to the discord api to create message
/// This could be the [`create_message`][crate::Api::create_message] endpoint, or `TODO: MORE ENDPOINTS`
#[derive(Serialize, Default, Debug)]
pub struct CreateMessageParams {
    /// Content to send
    pub content: Option<String>
}