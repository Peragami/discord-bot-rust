use std::{env, sync::Arc};

use anyhow::Result;
use twilight_gateway::{
    Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::{
        command::{Command, CommandType},
        interaction::{Interaction, InteractionData},
    },
    http::interaction::{
        InteractionResponse, InteractionResponseData, InteractionResponseType,
    },
    id::Id,
};

#[tokio::main]
async fn main() -> Result<()> {
    let token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN not set");

    let http = Arc::new(HttpClient::new(token.clone()));

    // ===== アプリID取得 =====
    let app = http.current_user_application().await?.model().await?;
    let application_id = app.id;

    // ===== Slash Command 登録 =====
    http.interaction(application_id)
        .set_global_commands(&[Command {
            id: None,
            application_id: None,
            guild_id: None,
            name: "rust".into(),
            description: "Rust bot is alive".into(),
            options: vec![],
            kind: CommandType::ChatInput,
            default_member_permissions: None,
            dm_permission: None, // ← これが必要
            nsfw: None,
            integration_types: None,
            contexts: None,
            version: Id::new(1),
            name_localizations: None,
            description_localizations: None,
        }])
        .await?;

    println!("Slash command registered.");

    // ===== Gateway =====
    let intents = Intents::empty();
    let mut shard = Shard::new(ShardId::ONE, token, intents);

    println!("Bot started.");

    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let event = event?;

        if let Event::InteractionCreate(boxed) = event {
            let interaction: Interaction = boxed.0;

            if let Some(InteractionData::ApplicationCommand(data)) = interaction.data {
                if data.name == "rust" {
                    http.interaction(application_id)
                        .create_response(
                            interaction.id,
                            &interaction.token,
                            &InteractionResponse {
                                kind: InteractionResponseType::ChannelMessageWithSource,
                                data: Some(InteractionResponseData {
                                    content: Some("🦀 Rust bot running on Debian Docker".into()),
                                    ..Default::default()
                                }),
                            },
                        )
                        .await?;
                }
            }
        }
    }

    Ok(())
}