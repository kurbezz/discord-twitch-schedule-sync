FROM rust:bullseye AS builder

WORKDIR /app

COPY . .

RUN cargo build --release --bin discord-twitch-schedule-sync


FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/discord-twitch-schedule-sync /usr/local/bin
ENTRYPOINT ["/usr/local/bin/discord-twitch-schedule-sync"]
