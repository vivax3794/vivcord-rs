const TOKEN: &str = "...";

#[tokio::main]
async fn main() {
    let client = vivcord::ApiClient::new(String::from(TOKEN));
    let url = client.get_gateway_url().await.unwrap();
    println!("{url}");
}