FROM rust:1-bookworm AS builder
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev
RUN rm -rf /var/lib/apt/lists/*

COPY . .
# `migrate!()` embeds ./migrations at compile time; runtime queries are unchecked
# (`sqlx::query`), so no live DB or SQLX_OFFLINE cache is needed to build.
RUN cargo build --release --bin discord --bin commands

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/discord ./discord
COPY --from=builder /build/target/release/commands ./commands

# config.json is mounted at runtime; the SQLite DB lives on a volume (DATABASE_URL).
ENV DISCORD_CONFIG_PATH=/app/config.json
CMD ["./discord"]
