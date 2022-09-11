//! The client ties together multiple parts of the crate into one common place.

use futures::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::EventData;

/// The client holds both a [`Api`][crate::Api] and a [`Gateway`][crate::Gateway]
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Client<S> {
    /// REST api interface
    pub api: Arc<RwLock<crate::Api>>,
    /// Websocket interface
    pub gateway: Arc<RwLock<crate::Gateway>>,
    
    /// Holds the global client state
    pub state: S,

    token: String,
}

impl<S> Client<S>
where
    S: Clone,
{
    /// Create a instance of client and
    #[must_use]
    pub fn new(token: String, state: S) -> Self {
        Self {
            api: Arc::new(RwLock::new(crate::Api::new(&token))),
            gateway: Arc::new(RwLock::new(crate::Gateway::new())),

            token,
            state,
        }
    }

    /// Run the bot
    ///
    /// # Panics
    /// panics if bot fails to connect to discord, might happen due to invalid oauth token.
    pub async fn run<F, A>(self, intents: &crate::Intents, event_callback: F)
    where
        F: Fn(EventData, Self) -> A + Send + 'static,
        A: Future<Output = ()> + Send + 'static,
    {
        let websocket_url = self.api.read().await.get_gateway_url().await.unwrap();

        // Limit scope of write lock
        {
            self.gateway
                .write()
                .await
                .connect(&websocket_url, &self.token, intents)
                .await;
        }

        self.gateway
            .read()
            .await
            .on(self.clone(), |event, client| {
                event_callback(event, client)
            })
            .await;
    }
}
