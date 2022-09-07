//! Code for interacting with the discord REST api

use serde::{Deserialize, Serialize};

/// Base url of discord api requests
const BASE_URL: &str = "https://discord.com/api/v10/";

/// Api client making requests to discord.
pub struct ApiClient {
    /// Internal http client used to make requests
    http_client: reqwest::Client,
}

impl ApiClient {
    /// Create new api client instance with the specified oauth token
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
    pub async fn get_gateway_url(&self) -> Result<String, reqwest::Error> {
        #[derive(Deserialize)]
        struct GatewayResponse {
            url: String
        }

        let result: GatewayResponse = self.http_client.get(format!("{BASE_URL}/gateway"))
            .send()
            .await?
            .json()
            .await?;

        Ok(result.url)
    }
}