const TOKEN: &str = include_str!("../token.secret");

#[tokio::main]
async fn main() {
    let api = vivcord::ApiClient::new(TOKEN);
    let url = api.get_gateway_url().await.unwrap();

    vivcord::Gateway::new().connect(&url, TOKEN).await;
}
