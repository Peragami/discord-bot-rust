use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::Client as HttpClient;
use twilight_model::{
    application::{
        command::CommandType,
        interaction::{Interaction, InteractionData},
    },
    application::interaction::application_command::CommandOptionValue,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::Id,
};
use twilight_util::builder::command::{CommandBuilder, StringBuilder};

// Python版 my-discord-bot のスラッシュコマンドをそのまま移植したもの。
// /hello /help /ping /echo /omikuji

static RNG_STATE: AtomicU64 = AtomicU64::new(0);

fn next_random_index(max: usize) -> usize {
    // おみくじ用なので暗号学的である必要はない。xorshift64* で十分。
    let mut x = RNG_STATE.load(Ordering::Relaxed);
    if x == 0 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        x = nanos ^ (std::process::id() as u64).wrapping_mul(0x2545_F491_4F6C_DD1D);
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
    }
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    RNG_STATE.store(x, Ordering::Relaxed);
    let v = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (v % (max as u64)) as usize
}

fn global_commands() -> Vec<twilight_model::application::command::Command> {
    vec![
        CommandBuilder::new("hello", "挨拶を返します", CommandType::ChatInput).build(),
        CommandBuilder::new("help", "ヘルプ", CommandType::ChatInput).build(),
        CommandBuilder::new("ping", "Botの応答速度を確認します", CommandType::ChatInput).build(),
        CommandBuilder::new("echo", "入力した文字をBotが復唱します", CommandType::ChatInput)
            .option(
                StringBuilder::new("message", "Botに喋らせたい文字").required(true),
            )
            .build(),
        CommandBuilder::new("omikuji", "今日の運勢を占います", CommandType::ChatInput).build(),
    ]
}

async fn respond(
    http: &HttpClient,
    application_id: Id<twilight_model::id::marker::ApplicationMarker>,
    interaction: &Interaction,
    content: String,
) {
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content),
            ..Default::default()
        }),
    };
    if let Err(e) = http
        .interaction(application_id)
        .create_response(interaction.id, &interaction.token, &response)
        .await
    {
        eprintln!("failed to respond to interaction: {e}");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // rustls の暗号プロバイダを明示的に固定する（未指定だと環境依存で panic することがある）
    let _ = rustls::crypto::ring::default_provider().install_default();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let http = Arc::new(HttpClient::new(token.clone()));

    // アプリID取得
    let app = http.current_user_application().await?.model().await?;
    let application_id = app.id;

    // グローバルコマンドを同期（Python版の tree.sync() 相当）
    http.interaction(application_id)
        .set_global_commands(&global_commands())
        .await?;
    println!("グローバルコマンドの同期が完了しました");

    // スラッシュコマンドだけなら intent は不要（インタラクションは intent なしで届く）
    let intents = Intents::empty();
    let event_types = EventTypeFlags::READY | EventTypeFlags::INTERACTION_CREATE;

    // Gateway が切れたら張り直す
    loop {
        let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

        while let Some(event) = shard.next_event(event_types).await {
            let event = match event {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("gateway error: {e}");
                    continue;
                }
            };

            match event {
                Event::Ready(ready) => {
                    println!("Logged in as {} (ID: {})", ready.user.name, ready.user.id);
                    println!("------");
                }
                Event::InteractionCreate(boxed) => {
                    let interaction: Interaction = boxed.0;
                    let Some(InteractionData::ApplicationCommand(ref data)) = interaction.data
                    else {
                        continue;
                    };

                    match data.name.as_str() {
                        "hello" => {
                            respond(
                                &http,
                                application_id,
                                &interaction,
                                "Hello! スラッシュコマンドが成功したよ！".to_owned(),
                            )
                            .await;
                        }
                        "help" => {
                            respond(
                                &http,
                                application_id,
                                &interaction,
                                "t!pで再生、キューの追加。t!sで再生停止、t!nでスキップ！".to_owned(),
                            )
                            .await;
                        }
                        "ping" => {
                            let latency_ms = shard
                                .latency()
                                .average()
                                .map(|d| d.as_millis())
                                .unwrap_or(0);
                            respond(
                                &http,
                                application_id,
                                &interaction,
                                format!("Pong! 遅延: {latency_ms}ms"),
                            )
                            .await;
                        }
                        "echo" => {
                            let message = data
                                .options
                                .iter()
                                .find(|o| o.name == "message")
                                .and_then(|o| match &o.value {
                                    CommandOptionValue::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .unwrap_or_default();
                            respond(
                                &http,
                                application_id,
                                &interaction,
                                format!("あなたが言ったこと: {message}"),
                            )
                            .await;
                        }
                        "omikuji" => {
                            const FORTUNES: [&str; 6] =
                                ["大吉", "中吉", "小吉", "吉", "末吉", "凶"];
                            let result = FORTUNES[next_random_index(FORTUNES.len())];
                            respond(
                                &http,
                                application_id,
                                &interaction,
                                format!("今日のあなたの運勢は… **{result}** です！"),
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        eprintln!("gateway stream ended; reconnecting in 5s");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
