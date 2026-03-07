FROM rust:1-bookworm AS builder
WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libpq-dev
RUN rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin backend

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libpq5
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/backend ./backend

EXPOSE 3060
CMD ["./backend"]
