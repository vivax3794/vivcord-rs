//! Connect and receive events from discord

// TODO: Reconnect on disconnect

mod events;
use std::{sync::Arc, time::Duration};

pub use events::EventData;

use futures::{Future, SinkExt, StreamExt};
use std::sync::Mutex;
use tokio::select;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Error, Message};

/// Same as [`Gateway::wait_for`] but operates on a stream.
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

/// Wait for specific event from gateway
///
/// For more control see [`Gateway::wait_for`][crate::Gateway::wait_for]
///
/// syntax is `wait_for!(gateway, pattern => return)`, you can use the pattern to capture values from the event to return.
///
/// # Example
/// ```no_run
/// # use vivcord::{wait_for, Gateway, EventData};
/// # tokio_test::block_on(async move {
/// let gateway = Gateway::new();
/// // IMPORTANT: normally you would have called `Gateway::connect` by this point!!!!!
/// let msg = wait_for!(gateway, EventData::MessageCreate(msg) => msg).await;
/// # });
/// ```
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

/// Same as [`wait_for!`], but operator on a stream instead.
///
/// Internal use only
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

/// Create the tls config for the gateway connection
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

/// Create the websocket connection using the given url.
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
#[derive(Debug)]
pub struct Gateway {
    event_reader: Option<broadcast::Receiver<EventData>>,
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}

impl Gateway {
    /// Create gateway instance.
    #[must_use]
    pub fn new() -> Self {
        Self { event_reader: None }
    }

    /// Create new gateway connection using a oauth token. <br>
    /// you can get the gateway url with [`ApiClient::get_gateway_url`](crate::Api::get_gateway_url) <br>
    /// This will spawn the event loop in a separate task (and maybe thread)
    ///
    /// # Panics
    /// If there is an error connecting to the gateway.
    #[allow(unreachable_code)]
    pub async fn connect(&mut self, url: &str, token: &str, intents: &crate::Intents) {
        let (mut stream_writer, stream_reader) = create_connection(url).await;

        // IMPORTANT: Should this be larger/smaller?
        // In theory all events should be processed almost at once
        // as long as the user doesn't block the thread (HEY MATISSE, SOUNDS FAMILIAR?)
        let (event_writer, event_reader) = broadcast::channel::<events::EventData>(5);
        self.event_reader = Some(event_reader);

        // create sequence number with Mutex so the event reader and heartbeat can both use it
        // we don't store this on the struct because it should only be useful to those 2 tasks
        // (this might change in the future when it comes to reconnecting lost connections)
        let sequence_number = Arc::new(Mutex::new(None));

        tokio::spawn(Gateway::event_loop(
            stream_reader,
            event_writer,
            sequence_number.clone(), // lets event reader thread update sequence number
        ));
        let hearth_interval =
            wait_for!(self, EventData::Hello { heartbeat_interval } => heartbeat_interval).await;

        // Send identify packet
        // Api allows us (and actually tells us) to send heartbeat's while doing this
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

        tokio::spawn(Gateway::heartbeat(
            stream_writer,
            self.event_reader.as_ref().unwrap().resubscribe(),
            hearth_interval,
            sequence_number,
        ));
    }

    /// Read events from socket forever
    async fn event_loop<S>(
        mut reader: S,
        event_writer: broadcast::Sender<EventData>,
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

    // Sent heartbeat to discord
    async fn heartbeat<W>(
        mut writer: W,
        mut event_reader: broadcast::Receiver<EventData>,
        interval: u32,
        sequence_number: Arc<Mutex<Option<u32>>>,
    ) -> !
    where
        W: SinkExt<Message, Error = Error> + Unpin,
    {
        // wait before sending intervals
        let first_sleep_amount = f64::from(interval) * rand::random::<f64>();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        tokio::time::sleep(Duration::from_millis(
            first_sleep_amount.floor().abs() as u64
        ))
        .await;

        loop {
            // send heartbeat event
            println!("sending heartbeat");
            let data = serde_json::json!({
                "op": 1,
                "d": sequence_number.lock().unwrap().clone()
            });
            let message = Message::Text(serde_json::to_string(&data).unwrap());
            writer.send(message).await.unwrap();

            // wait for response and timeout if it doesn't come
            // ... lets assume that one a good day discord wont be slow at responding.
            // and if we are being way to slow discord would ask us for a hearth anyway :D ❤️
            tokio::time::timeout(
                Duration::from_millis(interval.into()),
                wait_for_S!(&mut event_reader, EventData::HearthBeatAck => ()),
            )
            .await
            .expect("did not get response from discord in time!");

            // Send next heartbeat after interval milliseconds
            // or as soon as possible when a HeartbeatRequests comes from discord
            let sleeper_task = tokio::time::sleep(Duration::from_millis(interval.into()));
            let requests_waiter = wait_for_S!(&mut event_reader, EventData::HeartbeatRequest => ());

            // Wait for one of those tasks to finish, dropping (canceling) the other.
            select! {
                _ = sleeper_task => (),
                _ = requests_waiter => ()
            };
        }
    }

    /// Wait for event using predicate.
    ///
    /// This will call the predicate on each event gotten,
    /// once the predicate returns a `Some(value)` this function will then return the `value`,
    /// if the passed event is not the desired one return [`None`]
    ///
    /// # Example
    /// ```no_run
    /// # use vivcord::{Gateway, EventData};
    /// # tokio_test::block_on(async move {
    /// let gateway = Gateway::new();
    /// // IMPORTANT: you need to call `Gateway::connect` before using this function
    /// // For the sake of this example we have chosen to not do that
    /// let msg = gateway.wait_for(|event| {
    ///     if let EventData::MessageCreate(msg) = event {
    ///         Some(msg)
    ///     } else {
    ///         None
    ///     }
    /// }).await;
    /// # });
    /// ```
    pub async fn wait_for<T, F>(&self, predicate: F) -> T
    where
        F: FnMut(EventData) -> Option<T>,
    {
        let mut reader = self
            .event_reader
            .as_ref()
            .expect("Event loop not started")
            .resubscribe();
        wait_for(&mut reader, predicate).await
    }

    /// Keep calling `callback` with events gotten forever,
    /// This function never returns.
    ///
    /// The passed in state will be [cloned][Clone] and sent to each callback,
    /// consider using a [Mutex][std::sync::Mutex] to share data between callbacks.
    ///
    /// You can also define a struct to hold multiple Mutexes, to make the code more efficient (the less data behind a single lock the better);
    ///
    /// # Panics
    /// When the event loop has not been started yet, or if there is an error with reading the events from the event loop
    ///
    /// # Example
    /// ```no_run
    /// # use vivcord::Gateway;
    /// # tokio_test::block_on(async move {
    /// let gateway = Gateway::new();
    /// // As usual, you need to call `Gateway::connect` before this
    /// gateway.on(132, |event, state| async move {
    ///     assert_eq!(state, 123);
    ///     println!("{event:?}");
    /// });
    /// # })
    /// ```
    pub async fn on<F, A, S>(&self, state: S, mut callback: F) -> !
    where
        F: FnMut(EventData, S) -> A,
        A: Future<Output = ()> + Send + 'static,
        S: Clone,
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
