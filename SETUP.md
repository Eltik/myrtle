# Setup

## Assets
The following is a basic example on how to setup the assets downloader and unpacker.
1. Run `node build.js` in `assets/build.js`.
2. Download assets using:
```bash
cd assets/downloader
cargo run --release -- --server official --savedir ../ArkAssets
```
3. Extract assets using:
```bash
cd assets/unpacker

./run_unpacker.sh
```
You will need approximately ~100GB
