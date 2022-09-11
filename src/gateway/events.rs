//! Discord Gateway event handling

use serde::Deserialize;


/// Discord Events, returned from [`Gateway`][crate::Gateway]
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_name", content = "data")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum EventData {
    /// Sent when connecting to websocket
    /// 
    /// This is usually sent before even callbacks are registered
    #[serde(rename = "10")]
    Hello {
        heartbeat_interval: u32,
    },

    /// Sent to confirm we are still connected to websocket
    /// 
    /// This is handled by the internal event handler
    #[serde(rename = "11")]
    HearthBeatAck,

    /// Discord us wants us to verify we are still connected
    /// 
    /// This is handled by the internal event handler
    #[serde(rename = "1")]
    HeartbeatRequest,

    // TODO: More fields
    /// Sent when the client has successfully connected.
    Ready {},

    /// Send when somebody sends a message
    /// 
    /// # Important
    /// This is also sent when the bot creates a message, make sure to avoid infinite loops!
    MessageCreate(crate::datatypes::Message),
}

/// Raw data from discord api, used to convert into [`GatewayEvent`]
#[derive(Deserialize)]
struct RawEventData {
    #[serde(rename = "op")]
    opcode: u8,
    #[serde(rename = "t")]
    event_name: Option<String>,
    #[serde(rename = "s")]
    sequence_number: Option<u32>,
    #[serde(rename = "d")]
    data: Option<serde_json::Value>,
}

/// Stores general event data
#[derive(Deserialize, Debug, Clone)]
#[serde(from = "RawEventData")]
pub struct GatewayEvent {
    pub data: EventData,
    pub sequence_number: Option<u32>
}


// To make event parsing easier we make a custom pre-processing step to combine the opcode and event name into one field
// this lets us use the same enum for all event types!


impl From<RawEventData> for GatewayEvent {
    fn from(raw_event: RawEventData) -> Self {
        let event_name = if raw_event.opcode == 0 { raw_event.event_name.expect("Missing type field") } else { raw_event.opcode.to_string() };
        
        let data: EventData = serde_json::from_value(serde_json::json!({
            "event_name": event_name,
            "data": raw_event.data
        })).unwrap();

        GatewayEvent {
            data,
            sequence_number: raw_event.sequence_number
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let event: GatewayEvent = serde_json::from_str("{\"op\": 11}").unwrap();

        assert!(matches!(event.data, EventData::HearthBeatAck));
    }
    
    #[test]
    fn test_hello() {
        let event: GatewayEvent = serde_json::from_str("{\"op\": 10, \"d\": {\"heartbeat_interval\": 45000}}").unwrap();
        let data = event.data;

        if let EventData::Hello { heartbeat_interval } = data {
            assert_eq!(heartbeat_interval, 45000);
        } else {
            panic!("Expected Hello Event got {:?}", data);
        }
    }
}
