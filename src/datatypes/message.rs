use serde::{Deserialize, Serialize};

use super::Snowflake;
use crate::to_snowflake_simple;

#[derive(Deserialize, Clone)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub content: String,
}

to_snowflake_simple!(Message);

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("content", &self.content)
            .finish_non_exhaustive()
    }
}

// Used for create message endpoints in api
#[derive(Serialize, Default)]
// #[non_exhaustive]
pub struct CreateMessageParams {
    pub content: Option<String>
}