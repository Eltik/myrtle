# Version Checking System

A scalable version checking system for monitoring Arknights game and asset updates.

## Overview

The version checking system allows you to:
- Monitor for game client and resource updates automatically
- Cache version information locally for comparison
- Detect exactly what changed (client, resources, or both)
- Integrate into production workflows (CI/CD, monitoring, auto-updates)

## How It Works

1. **Fetch Current Version**: Queries the Arknights server for the latest version info
2. **Compare with Cache**: Loads cached version from `version_cache_{server}.json`
3. **Detect Changes**: Compares resource version and client version
4. **Update Cache**: Automatically updates cache when changes are detected

## Version Cache Format

The system stores version info in JSON format at `{savedir}/version_cache_{server}.json`:

```json
{
  "resVersion": "24-02-02-10-18-07-831840",
  "clientVersion": "1.9.51",
  "lastChecked": "2024-02-15T14:30:00Z"
}
```

## API Reference

### `check_for_updates()`

Check for version updates by comparing server version with cached version.

```rust
pub fn check_for_updates(
    server: Servers,
    savedir: &str,
) -> Result<UpdateStatus, Box<dyn std::error::Error>>
```

**Parameters:**
- `server` - Server to check (OFFICIAL or BILIBILI)
- `savedir` - Directory where version cache is stored

**Returns:**
- `UpdateStatus` - Enum indicating what changed (if anything)

**Example:**
```rust
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

match ArkAssets::check_for_updates(Servers::OFFICIAL, "./ArkAssets") {
    Ok(UpdateStatus::NoUpdate) => println!("Up to date!"),
    Ok(UpdateStatus::ResourceUpdate { old_version, new_version }) => {
        println!("Resource update: {} -> {}", old_version, new_version);
    }
    Err(e) => eprintln!("Error: {}", e),
    _ => {}
}
```

### `load_version_cache()`

Load cached version information from disk.

```rust
pub fn load_version_cache(server: Servers, savedir: &str) -> Option<VersionInfo>
```

**Returns:**
- `Some(VersionInfo)` - Cached version data if it exists
- `None` - If no cache file exists or parsing failed

**Example:**
```rust
if let Some(cached) = ArkAssets::load_version_cache(Servers::OFFICIAL, "./ArkAssets") {
    println!("Current resource version: {}", cached.resVersion);
    println!("Last checked: {}", cached.lastChecked);
}
```

### `save_version_cache()`

Manually save version information to cache (usually not needed - `check_for_updates` does this automatically).

```rust
pub fn save_version_cache(
    server: Servers,
    savedir: &str,
    res_version: &str,
    client_version: &str,
) -> Result<(), Box<dyn std::error::Error>>
```

## Update Status Types

The `UpdateStatus` enum has 5 variants:

### `NoUpdate`
No changes detected - you're on the latest version.

```rust
UpdateStatus::NoUpdate
```

### `FirstCheck`
No cached version exists - this is the first check. A baseline has been established.

```rust
UpdateStatus::FirstCheck
```

### `ResourceUpdate`
Only the resource (asset) version changed.

```rust
UpdateStatus::ResourceUpdate {
    old_version: String,  // Previous resource version
    new_version: String   // New resource version
}
```

**Action Required:** Re-download assets to get updated game data.

### `ClientUpdate`
Only the game client version changed.

```rust
UpdateStatus::ClientUpdate {
    old_version: String,  // Previous client version
    new_version: String   // New client version
}
```

**Action Required:** Usually informational - assets may not need updating.

### `BothUpdated`
Both resource and client versions changed (major update).

```rust
UpdateStatus::BothUpdated {
    old_res_version: String,     // Previous resource version
    new_res_version: String,     // New resource version
    old_client_version: String,  // Previous client version
    new_client_version: String   // New client version
}
```

**Action Required:** Major update - recommend full asset re-download.

## Usage Examples

### Basic Version Check

```rust
use arknights_downloader::downloader::{ArkAssets, Servers};

fn main() {
    match ArkAssets::check_for_updates(Servers::OFFICIAL, "./ArkAssets") {
        Ok(status) => println!("Update status: {:?}", status),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Production Monitoring Script

```rust
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

fn check_and_notify() -> Result<(), Box<dyn std::error::Error>> {
    let status = ArkAssets::check_for_updates(Servers::OFFICIAL, "./data")?;

    match status {
        UpdateStatus::NoUpdate => {
            // Nothing to do
            Ok(())
        }
        UpdateStatus::ResourceUpdate { new_version, .. } => {
            // Send notification (Slack, Discord, email, etc.)
            send_notification(&format!("New resource version: {}", new_version))?;

            // Optionally trigger auto-download
            trigger_download()?;

            Ok(())
        }
        _ => Ok(())
    }
}
```

### Automated Downloader with Update Detection

```rust
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

fn auto_update_loop() {
    loop {
        match ArkAssets::check_for_updates(Servers::OFFICIAL, "./ArkAssets") {
            Ok(UpdateStatus::ResourceUpdate { .. }) |
            Ok(UpdateStatus::BothUpdated { .. }) => {
                println!("Update detected - downloading...");

                if let Ok(assets) = ArkAssets::new(Servers::OFFICIAL) {
                    let all_packages: Vec<String> = assets
                        .hot_update_list
                        .keys()
                        .cloned()
                        .collect();

                    let _ = assets.download_fromlist(&all_packages, "./ArkAssets", 6);
                }
            }
            _ => {}
        }

        // Check every hour
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
```

## Running the Examples

### Simple Version Checker
```bash
cargo run --example check_updates
```

### Production Monitor with Exit Codes
```bash
# Default configuration
cargo run --example automated_monitor

# Custom configuration via environment variables
ARK_SERVER=official \
ARK_SAVEDIR=./production_assets \
ARK_AUTO_DOWNLOAD=true \
cargo run --example automated_monitor
```

**Exit Codes:**
- `0` - No updates / Download successful
- `1` - Error occurred
- `2` - First check (baseline established)
- `10` - Resource update detected
- `11` - Client update detected
- `12` - Both resource and client updated

### Cron Job Integration

Add to your crontab for hourly checks:

```bash
# Check for updates every hour
0 * * * * cd /path/to/project && /path/to/cargo run --example automated_monitor >> /var/log/ark-monitor.log 2>&1
```

### Docker/CI Integration

```dockerfile
FROM rust:latest

WORKDIR /app
COPY . .

RUN cargo build --release --example automated_monitor

# Run every 6 hours
CMD while true; do \
    ./target/release/examples/automated_monitor; \
    sleep 21600; \
done
```

## Production Best Practices

1. **Error Handling**: Always wrap version checks in proper error handling
2. **Rate Limiting**: Don't check too frequently (recommended: every 30-60 minutes)
3. **Logging**: Log all update checks with timestamps
4. **Notifications**: Integrate with your notification system (Slack, Discord, etc.)
5. **Monitoring**: Track exit codes for alerting
6. **Storage**: Ensure `savedir` is persistent storage (not /tmp)

## Troubleshooting

### Cache file not updating
- Check file permissions on `{savedir}/version_cache_{server}.json`
- Ensure `savedir` directory exists and is writable

### Always returns FirstCheck
- Verify the cache file exists at the correct path
- Check JSON format is valid

### Network errors
- Verify internet connectivity
- Check if Arknights servers are accessible
- Ensure no firewall blocking requests to `ak-conf.hypergryph.com`

## Implementation Details

### Version Cache Location
- **Official Server**: `{savedir}/version_cache_official.json`
- **Bilibili Server**: `{savedir}/version_cache_bilibili.json`

### Network Requests
- Queries: `https://ak-conf.hypergryph.com/config/prod/{server}/Android/version`
- Response format: `{"resVersion": "...", "clientVersion": "..."}`
- Timeout: Uses default reqwest timeout
- User-Agent: Not set for version checks (uses reqwest default)

### Thread Safety
All methods are thread-safe and can be called concurrently from multiple threads.

### Cache Timestamp
The `lastChecked` field uses ISO 8601 format (RFC 3339) in UTC timezone.
