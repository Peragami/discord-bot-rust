# ===== builder =====
FROM rust:1.75-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# ===== runtime =====
FROM debian:bookworm

WORKDIR /app
COPY --from=builder /app/target/release/discord-bot-rust .

CMD ["./discord-bot-rust"]