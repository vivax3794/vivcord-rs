//! Connect and recive events from discord

mod events;
use std::{sync::Arc, time::Duration};

pub use events::GatewayEventData;

use futures::{Future, SinkExt, StreamExt};
use std::sync::Mutex;
use tokio::select;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Error, Message};

async fn wait_for<St, F, Rt>(stream: &mut broadcast::Receiver<St>, mut predicate: F) -> Rt
where
    St: Clone,
    F: FnMut(St) -> Option<Rt>,
{
    loop {
        if let Ok(message) = stream.recv().await {
            if let Some(result) = predicate(message) {
                return result;
            }
        }
    }
}

#[macro_export]
macro_rules! wait_for {
    ($gateway: expr, $event: pat => $return_expr: expr) => {
        $gateway.wait_for(|event| {
            if let $event = event {
                Some($return_expr)
            } else {
                None
            }
        })
    };
}

macro_rules! wait_for_S {
    ($stream: expr, $event: pat => $return_expr: expr) => {
        wait_for($stream, |event| {
            if let $event = event {
                Some($return_expr)
            } else {
                None
            }
        })
    };
}

fn create_tls() -> tokio_tungstenite::Connector {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let tls = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    tokio_tungstenite::Connector::Rustls(Arc::new(tls))
}

async fn create_connection(
    url: &str,
) -> (
    impl SinkExt<Message, Error = Error>,
    impl StreamExt<Item = Result<Message, Error>>,
) {
    let url = format!("{url}?v=10&encoding=json");
    let url = reqwest::Url::parse(&url).unwrap();

    let config = WebSocketConfig::default();
    let tls = create_tls();

    let (connection, _) =
        tokio_tungstenite::connect_async_tls_with_config(url, Some(config), Some(tls))
            .await
            .unwrap();
    connection.split()
}

/// Websocket for getting events from discord gateway.
pub struct Gateway {
    event_reader: Option<broadcast::Receiver<GatewayEventData>>,
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}

impl Gateway {
    pub fn new() -> Self {
        Self { event_reader: None }
    }

    /// Create new gateway connection using a oauth token
    /// you can get the gateway url with [ApiClient::get_gateway_url](crate::ApiClient::get_gateway_url)
    /// This will spawn the event loop in a seperate task (and maybe thread)
    #[allow(unreachable_code)]
    pub async fn connect(&mut self, url: &str, token: &str, intents: &crate::Intents) {
        let (mut stream_writer, stream_reader) = create_connection(url).await;

        // IMPORTANT: Should this be larger/smaller?
        // In theory all events should be processed almost at once
        // as long as the user doesnt block the thread (HEY MATISSE, SOUNDS FAMILIAR?)
        let (event_writer, event_reader) = broadcast::channel::<events::GatewayEventData>(5);
        self.event_reader = Some(event_reader);

        let sequence_number = Arc::new(Mutex::new(None));

        tokio::spawn(Gateway::event_loop(
            stream_reader,
            event_writer,
            sequence_number.clone(), // lets event reader thread update sequence number
        ));
        let hearth_interval =
            wait_for!(self, GatewayEventData::Hello { heartbeat_interval } => heartbeat_interval)
                .await;

        // Send identify packet
        // Api allows us (and actually tells us) to send hearthbeats while doing this
        // but that would make the heartbeat logic very complicated since we cant copy the stream writer
        let data = Message::Text(
            serde_json::to_string(&serde_json::json!({
                "op": 2,
                "d": {
                    "token": token,
                    "intents": intents.bits(),
                    "properties": {
                        "os": std::env::consts::OS,
                        "browser": "vivcord-rs",
                        "device": "vivcord-rs"
                    }
                }
            }))
            .unwrap(),
        );
        stream_writer.send(data).await.unwrap();

        tokio::spawn(Gateway::hearthbeat(
            stream_writer,
            self.event_reader.as_ref().unwrap().resubscribe(),
            hearth_interval,
            sequence_number,
        ));
    }

    /// Read events from socket forever
    async fn event_loop<S>(
        mut reader: S,
        event_writer: broadcast::Sender<GatewayEventData>,
        sequence_number: Arc<Mutex<Option<u32>>>,
    ) -> !
    where
        S: StreamExt<Item = Result<Message, Error>> + Unpin,
    {
        loop {
            let msg = reader.next().await.unwrap().unwrap();
            match msg {
                Message::Text(data) => {
                    let event: events::GatewayEvent = serde_json::from_str(&data).unwrap();
                    eprintln!("got event {:?}", event.data);
                    *sequence_number.lock().unwrap() = event.sequence_number;

                    // Ok => just amount it was sent to
                    // Err => nobody is listening, but they might in the future
                    if event_writer.send(event.data).is_err() {
                        eprintln!("[WARNING] Nobody listening to value");
                    }
                }
                _ => panic!("expected data to be text, got {msg:?}"),
            }
        }
    }

    async fn hearthbeat<W>(
        mut writer: W,
        mut event_reader: broadcast::Receiver<GatewayEventData>,
        interval: u32,
        sequence_number: Arc<Mutex<Option<u32>>>,
    ) -> !
    where
        W: SinkExt<Message, Error = Error> + Unpin,
    {
        // wait before sending intervals
        let first_sleep_amount = (interval as f64) * rand::random::<f64>();
        tokio::time::sleep(Duration::from_millis(first_sleep_amount as u64)).await;

        loop {
            // send hearthbeat event
            println!("sending hearthbeat");
            let data = serde_json::json!({
                "op": 1,
                "d": sequence_number.lock().unwrap().clone()
            });
            let message = Message::Text(serde_json::to_string(&data).unwrap());
            writer.send(message).await.unwrap();

            // wait for response and timeout if it doesnt come
            // TODO: make reconnection logic.... somehow?
            // ... lets assume that one a good day discord wont be slow at responding.
            // and if we are being way to slow discord would ask us for a hearth!
            tokio::time::timeout(
                Duration::from_millis(interval as u64),
                wait_for_S!(&mut event_reader, GatewayEventData::HearthBeatAck => ()),
            )
            .await
            .expect("did not get response from discord in time!");

            // sleep for interval, or until discord requests a heartbeat
            let sleeper_task = tokio::time::sleep(Duration::from_millis(interval as u64));
            let requests_waiter =
                wait_for_S!(&mut event_reader, GatewayEventData::HearthbeatRequest => ());

            select! {
                _ = sleeper_task => (),
                _ = requests_waiter => ()
            };
        }
    }

    pub async fn wait_for<T, F>(&self, predicate: F) -> T
    where
        F: FnMut(GatewayEventData) -> Option<T>,
    {
        let mut reader = self
            .event_reader
            .as_ref()
            .expect("Event loop not started")
            .resubscribe();
        wait_for(&mut reader, predicate).await
    }

    pub async fn on<F, A, S>(&self, state: S, mut callback: F) -> !
    where
        F: FnMut(GatewayEventData, S) -> A,
        A: Future<Output = ()> + Send + 'static,
        S: Clone
    {
        let mut reader = self
            .event_reader
            .as_ref()
            .expect("Event loop not started")
            .resubscribe();
        loop {
            let event = reader.recv().await.unwrap();
            tokio::spawn(callback(event, state.clone()));
        }
    }
}
