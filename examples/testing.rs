use std::sync::Arc;

const TOKEN: &str = include_str!("../token.secret");

#[tokio::main]
async fn main() {
    let api = vivcord::ApiClient::new(TOKEN);
    let url = api.get_gateway_url().await.unwrap();

    let intents = vivcord::Intents::GUILD_MESSAGES | vivcord::Intents::MESSAGE_CONTENT;

    let mut gateway = vivcord::Gateway::new();
    gateway.connect(&url, TOKEN, &intents).await;
    gateway
        .on(
            Arc::new(tokio::sync::Mutex::new(api)),
            |event, api| async move {
                if let vivcord::GatewayEventData::MessageCreate(msg) = event {
                    if msg.content == "test" {
                        api.lock()
                            .await
                            .create_message(
                                msg.channel_id,
                                vivcord::CreateMessageParams {
                                    content: Some("hello there!".to_owned()),
                                },
                            )
                            .await
                            .unwrap();
                    }
                }
            },
        )
        .await;
}
