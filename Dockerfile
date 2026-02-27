# ===== build stage =====
FROM rust:1.76-slim-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# ===== runtime stage =====
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/discord-bot-rust .

CMD ["./discord-bot-rust"]