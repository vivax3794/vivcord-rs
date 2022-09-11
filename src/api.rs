//! Code for interacting with the discord REST api

use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    datatypes::{Message, Snowflake},
    CreateMessageParams,
};

/// Error from the discord api
#[derive(Deserialize, Clone, Debug)]
pub struct DiscordErrorData {
    /// Json Structure describing exactly what was wrong
    pub errors: Option<serde_json::Value>,
    /// Generic message of what was wrong, usually not needed
    pub message: String,
    /// Discord [error code](https://discord.com/developers/docs/dispatch/error-codes#error-codes)
    pub code: u16,
}

/// Holds possible errors from the api
#[derive(Debug)]
pub enum ApiErr {
    /// Generic http error
    ReqwstErr(reqwest::Error),
    /// Error from discord api
    DiscordErr(DiscordErrorData),
}

impl From<reqwest::Error> for ApiErr {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwstErr(err)
    }
}

/// Parse a json that might be `T` or might be a discord error
fn parse_possible_error<T: DeserializeOwned>(data: serde_json::Value) -> Result<T, ApiErr> {
    if data.get("code").is_some() {
        Err(ApiErr::DiscordErr(serde_json::from_value(data).unwrap()))
    } else {
        Ok(serde_json::from_value(data).unwrap())
    }
}

/// Base url of discord api requests
const BASE_URL: &str = "https://discord.com/api/v10/";

/// Api client making requests to discord.
pub struct ApiClient {
    /// Internal http client used to make requests
    http_client: reqwest::Client,
}

impl ApiClient {
    /// Create new api client instance with the specified oauth token
    /// Takes a discord api oauth token.
    ///
    /// Even if not *all* endpoint technically require a oauth token, 99% does, so we require it to create our instance.
    pub fn new(token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(1);
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bot {token}")).expect("Invalid Token"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            http_client: client,
        }
    }

    /// Get the connection url for the discord gateway
    /// At the time of writing this url is most likely `wss://gateway.discord.gg/`, but this might change.
    ///
    /// # Example
    /// ```no_run
    /// # use vivcord::ApiClient;
    /// # tokio_test::block_on(async {
    /// let client = ApiClient::new("");
    /// let url = client.get_gateway_url().await?;
    /// # Ok::<(), reqwest::Error>(())
    /// # });
    /// ```

    pub async fn get_gateway_url(&self) -> Result<String, reqwest::Error> {
        #[derive(Deserialize)]
        struct GatewayResponse {
            url: String,
        }

        let result: GatewayResponse = self
            .http_client
            .get(format!("{BASE_URL}/gateway"))
            .send()
            .await?
            .json()
            .await?;

        Ok(result.url)
    }

    /// Send message to specific channel
    /// 
    /// # Example
    /// ```no_run
    /// # use vivcord::{ApiClient, CreateMessageParams, api::ApiErr};
    /// # tokio_test::block_on(async move {
    /// let api = ApiClient::new("TOKEN");
    /// api.create_message(12345, CreateMessageParams {content: Some("hello".to_owned())}).await?;
    /// # Ok::<(), ApiErr>(())
    /// # });
    /// ```
    pub async fn create_message<I: Into<Snowflake>>(
        &self,
        channel_id: I,
        msg: CreateMessageParams,
    ) -> Result<Message, ApiErr> {
        let id: u64 = channel_id.into().0;

        parse_possible_error(
            self.http_client
                .post(format!("{BASE_URL}/channels/{id}/messages"))
                .json(&msg)
                .send()
                .await?
                .json()
                .await?,
        )
    }
}
