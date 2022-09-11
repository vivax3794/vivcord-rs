//! Types used by discord

use bitflags::bitflags;

bitflags! {
    /// Intents tells discord what events you want to be passed. <br>
    /// You can use bitwise operations to combine flags. <br>
    /// These should be passed to [`Gateway::connect`][crate::Gateway::connect]
    /// 
    /// Detailed description of what each flag does can be seen in the [discord docs](https://discord.com/developers/docs/topics/gateway#list-of-intents)
    /// 
    /// # Example
    /// ```
    /// # use vivcord::Intents;
    /// // You want to get events about messages in guilds and their content.
    /// let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
    /// ```
    pub struct Intents: u32 {
        const GUILDS = 1 << 0;
        const GUILD_MEMBERS = 1 << 1;
        const GUILD_BANS = 1 << 2;
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        const GUILD_INTEGRATIONS = 1 << 4;
        const GUILD_WEBHOOKS = 1 << 5;
        const GUILD_INVITES = 1 << 6;
        const GUILD_VOICE_STATES = 1 << 7;
        const GUILD_PRESENCES = 1 << 8;
        const GUILD_MESSAGES = 1 << 9;
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        const GUILD_MESSAGE_TYPING = 1 << 11;
        const DIRECT_MESSAGES = 1 << 12;
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        const DIRECT_MESSAGE_TYPING = 1 << 14;
        const MESSAGE_CONTENT = 1 << 15;
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        const AUTO_MODERATION_EXECUTION = 1 << 21;
    }
}

mod message;
mod snowflake;

pub use message::{Message, CreateMessageParams};
pub use snowflake::Snowflake;