# ===== build stage =====
FROM rust:1.76-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml .
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN cargo build --release

# ===== runtime stage =====
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rust-bot .

CMD ["./rust-bot"]