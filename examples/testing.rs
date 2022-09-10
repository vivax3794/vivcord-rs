const TOKEN: &str = include_str!("../token.secret");

#[tokio::main]
async fn main() {
    let api = vivcord::ApiClient::new(TOKEN);
    let url = api.get_gateway_url().await.unwrap();


    let mut gateway = vivcord::Gateway::new();
    gateway.connect(&url, TOKEN).await;
    gateway.on(|event| async move {
        println!("CALLBACK TEST: {event:?}")
    }).await;
    
}
