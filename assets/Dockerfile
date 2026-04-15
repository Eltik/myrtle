FROM rust:1-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3-dev pkg-config libssl-dev git ca-certificates \
    build-essential unzip
RUN rm -rf /var/lib/apt/lists/*

RUN curl -LO https://github.com/google/flatbuffers/releases/download/v25.12.19/Linux.flatc.binary.g++-13.zip \
    && unzip Linux.flatc.binary.g++-13.zip -d /usr/local/bin/ \
    && chmod +x /usr/local/bin/flatc \
    && rm Linux.flatc.binary.g++-13.zip

WORKDIR /build
COPY . .

RUN --mount=type=cache,target=/build/downloader/target \
    cd downloader && cargo build --release && \
    cp target/release/downloader /usr/local/bin/

RUN --mount=type=cache,target=/build/unpacker/target \
    cd unpacker && cargo build --release && \
    cp target/release/unpacker /usr/local/bin/

RUN cp download.sh unpack.sh generate_fbs.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/download.sh /usr/local/bin/unpack.sh /usr/local/bin/generate_fbs.sh

RUN ldconfig

RUN mkdir -p /data/ArkAssets /data/output
VOLUME ["/data/ArkAssets", "/data/output"]

CMD ["sh", "-c", "echo 'Run with: download.sh or unpack.sh'"]
