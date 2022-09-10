//! Discord Gateway event handeling

use serde::Deserialize;


/// Stores all possible gateway event types.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_name", content = "data")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum GatewayEventData {
    /// Hello Packet, contains a hearthbeat describing the interval for the keep-alive loop!
    #[serde(rename = "10")]
    Hello {
        heartbeat_interval: u32,
    },
    #[serde(rename = "11")]
    HearthBeatAck,
    #[serde(rename = "1")]
    HearthbeatRequest,
    Ready {},
}

/// Raw data from discord api, used to convert into GatewayEvent
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
    pub data: GatewayEventData,
    pub sequence_number: Option<u32>
}


impl From<RawEventData> for GatewayEvent {
    fn from(raw_event: RawEventData) -> Self {
        let event_name = if raw_event.opcode == 0 { raw_event.event_name.expect("Missing type field") } else { raw_event.opcode.to_string() };
        
        let data: GatewayEventData = serde_json::from_value(serde_json::json!({
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

        assert!(matches!(event.data, GatewayEventData::HearthBeatAck));
    }
    
    #[test]
    fn test_hello() {
        let event: GatewayEvent = serde_json::from_str("{\"op\": 10, \"d\": {\"heartbeat_interval\": 45000}}").unwrap();
        let data = event.data;

        if let GatewayEventData::Hello { heartbeat_interval } = data {
            assert_eq!(heartbeat_interval, 45000);
        } else {
            panic!("Expected Hello Event got {:?}", data);
        }
    }
}
