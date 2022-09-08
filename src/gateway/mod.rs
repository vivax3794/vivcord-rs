//! Connect and recive events from discord

mod event;

pub use event::GatewayEventData;

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
/// Websocket for getting events from discord gateway.
pub struct Gateway {
    // websocket client
    client: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>
}

impl Gateway {
    /// Create new gateway connection using a oauth token
    /// you can get the gateway url with [ApiClient::get_gateway_url](crate::ApiClient::get_gateway_url)
    pub async fn connect(url: &str) -> Self {
        let url = format!("{url}?v=10&encoding=json");
        let config = WebSocketConfig::default();
        let tls = tokio_tungstenite::Connector::NativeTls(native_tls::TlsConnector::new().unwrap());

        let (connection, _) =
            tokio_tungstenite::connect_async_tls_with_config(url, Some(config), Some(tls))
                .await
                .unwrap();

        Self {
            client: connection
        }
    }
}
