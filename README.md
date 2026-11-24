# my-discord-bot (Rust版)

Python版 `my-discord-bot` のスラッシュコマンドを Rust（twilight）に移植したもの。

## コマンド

| コマンド | 内容 |
|---|---|
| `/hello` | 挨拶を返す |
| `/help` | 音楽botの prefix コマンド案内 |
| `/ping` | Gateway の遅延を表示 |
| `/echo message:` | 入力した文字を復唱 |
| `/omikuji` | 今日の運勢を占う |

## 必要なもの

- Rust 1.89 以上（twilight 0.17 の要件）
- 環境変数 `DISCORD_TOKEN`（この bot のトークン）

## ビルドと実行

```bash
cargo build --release
DISCORD_TOKEN=xxxx ./target/release/discord-bot-rust
```

## systemd で常駐させる例（CT106）

配置: `/opt/bots/live-rust/` にバイナリと `.env`（`DISCORD_TOKEN=...`）を置く。

```ini
[Unit]
Description=Discord bot live (Rust)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/bots/live-rust
EnvironmentFile=/opt/bots/live-rust/.env
ExecStart=/opt/bots/live-rust/discord-bot-rust
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## メモ

- スラッシュコマンドだけなので Gateway intent は空。起動時にグローバルコマンドを同期する。
- Gateway が切れたら 5 秒後に自動で張り直す。
- Python版と**同じ bot アカウントでは同時起動しない**こと（コマンド同期や応答が競合する）。
