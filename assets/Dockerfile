FROM rust:1-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3-dev pkg-config libssl-dev git ca-certificates
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/downloader/target \
    cd downloader && cargo build --release && \
    cp target/release/downloader /usr/local/bin/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/unpacker/target \
    cd unpacker && cargo build --release && \
    cp target/release/unpacker /usr/local/bin/

RUN cp download.sh unpack.sh /usr/local/bin/ 
RUN chmod +x /usr/local/bin/download.sh /usr/local/bin/unpack.sh

RUN cp unity-rs/src/resources/FMOD/libfmod.so /usr/local/lib/
RUN cp unity-rs/src/resources/FMOD/libfmodstudio.so /usr/local/lib/
RUN ldconfig

RUN mkdir -p /data/ArkAssets /data/output
VOLUME ["/data/ArkAssets", "/data/output"]
    
CMD ["sh", "-c", "echo 'Run with: download.sh or unpack.sh'"]
