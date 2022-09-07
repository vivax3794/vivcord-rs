//! Code for interacting with the discord REST api

use serde::Deserialize;

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
    /// Even if not *all* endpoint technically require a oauth token, 99% does, so we require it to create out instace.
    pub fn new(token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::with_capacity(1);
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&token).expect("Invalid Token"),
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
    /// At the time of writting this url is most likely `wss://gateway.discord.gg/`, but this might change.
    ///
    /// # Example
    /// ```no_run
    /// # use vivcord::ApiClient;
    /// # tokio_test::block_on(async {
    /// let client = ApiClient::new(String::from(""));
    /// let url = client.get_gateway_url().await?;
    ///     # Ok::<(), reqwest::Error>(())
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
}
