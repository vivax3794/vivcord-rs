const TOKEN: &str = include_str!("../token.secret");

use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(0));
    let client = vivcord::Client::new(TOKEN.to_owned(), count);

    let intents = vivcord::Intents::MESSAGE_CONTENT | vivcord::Intents::GUILD_MESSAGES;
    client
        .run(&intents, |event, client| async move {
            if let vivcord::EventData::MessageCreate(msg) = event {
                if msg.content == "test" {
                    let new_msg = {
                        let mut count = client.state.lock().unwrap();
                        *count += 1;

                        vivcord::CreateMessageParams {
                            content: Some(count.to_string()),
                        }
                    };

                    client
                        .api
                        .read()
                        .await
                        .create_message(msg.channel_id, new_msg)
                        .await
                        .unwrap();
                }
            }
        })
        .await;
}
