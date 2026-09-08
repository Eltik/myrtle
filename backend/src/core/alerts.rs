//! Out-of-band alerts for failures that are otherwise silent.
//!
//! The motivating case: `load_table_or_warn` catches a deserialization error,
//! logs one `tracing::warn!` line, and substitutes `T::default()`. The service
//! then serves an empty table as if it were real data — no error, no failing
//! health check, and a smoke test that passes. A single unrecognised enum value
//! in `item_table` is enough to blank every item name and icon on a server.
//!
//! Optional: with `DISCORD_ALERT_WEBHOOK` unset this module does nothing.

use serde_json::json;

/// Env var holding a Discord webhook URL. Unset disables alerting entirely.
const WEBHOOK_ENV: &str = "DISCORD_ALERT_WEBHOOK";

/// Post the degraded-table list for one server, if there is anything to say.
///
/// Fire-and-forget: a failed webhook is logged and swallowed. Alerting must
/// never be able to take down a reload.
pub async fn report_degraded_tables(client: &reqwest::Client, server: &str, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    tracing::warn!(
        server,
        count = warnings.len(),
        "game data loaded with degraded tables"
    );

    let Ok(webhook) = std::env::var(WEBHOOK_ENV) else {
        return;
    };
    if webhook.trim().is_empty() {
        return;
    }

    // Discord caps an embed description at 4096 characters; keep well clear and
    // say so rather than truncating silently.
    const MAX_LISTED: usize = 15;
    let mut lines: Vec<String> = warnings
        .iter()
        .take(MAX_LISTED)
        .map(|w| format!("• `{}`", w.replace('`', "'")))
        .collect();
    if warnings.len() > MAX_LISTED {
        lines.push(format!("…and {} more", warnings.len() - MAX_LISTED));
    }
    let description = lines.join("\n");

    let body = json!({
        "embeds": [{
            "title": format!("Degraded game-data tables ({server})"),
            "description": description,
            "color": 0x00E0_A81A_u32,
            "footer": { "text": "These tables fell back to empty defaults. Data served from them is wrong, not missing." }
        }]
    });

    match client.post(&webhook).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => tracing::error!(status = %resp.status(), "degraded-table alert rejected"),
        Err(e) => tracing::error!(error = %e, "degraded-table alert failed to send"),
    }
}
