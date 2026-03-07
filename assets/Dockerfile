FROM rust:1-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3-dev pkg-config libssl-dev git ca-certificates
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cd downloader && cargo build --release
RUN cd unpacker && cargo build --release

RUN cp downloader/target/release/arknights-downloader /usr/local/bin/
RUN cp unpacker/target/release/assets-unpacker /usr/local/bin/
RUN cp unity-rs/src/resources/FMOD/libfmod.so /usr/local/lib/
RUN cp unity-rs/src/resources/FMOD/libfmodstudio.so /usr/local/lib/
RUN ldconfig

RUN mkdir -p /data/ArkAssets /data/Unpacked
VOLUME ["/data/ArkAssets", "/data/Unpacked"]

CMD ["sh", "-c", "echo 'Run with: arknights-downloader --help  or  assets-unpacker --help'"]
