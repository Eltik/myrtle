// PM2 config for the asset watcher(s).
//
// One watcher process per region (the backend connects to each over its own WS
// port — see backend ASSET_WS_URLS, default en=9160 / cn=9161). Each polls the
// HG CDN on an interval and re-downloads + re-extracts when a new resVersion or
// a rebuilt unpacker is detected.
//
// `run.mjs ws` runs the WebSocket server non-interactively; the WS_* env vars
// fill in what the interactive prompts otherwise would. `cwd` must be the assets
// dir so the bundled `binaries/`, `./ArkAssets/<region>` and `./output/<region>`
// paths resolve (run.mjs joins the region onto the savedir/output itself).
module.exports = {
    apps: [
        {
            name: "myrtle-ws-en",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "en",
                WS_PORT: "9160",
                WS_PROFILE: "full",
            },
        },
        {
            name: "myrtle-ws-cn",
            script: "run.mjs",
            interpreter: "node",
            cwd: "/var/www/myrtle.moe/assets",
            args: "ws",
            max_memory_restart: "2G",
            env: {
                WS_SERVER: "cn",
                WS_PORT: "9161",
                WS_PROFILE: "operators",
            },
        },
    ],
};
