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
./target/release/assets-unpacker extract --input ../ArkAssets --output ../Unpacked
  ```
You will need approximately ~100GB