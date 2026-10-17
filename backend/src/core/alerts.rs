//! Out-of-band alerts for failures that are otherwise silent.
//!
//! - Degraded tables: `load_table_or_warn` catches a deserialization error,
//!   logs one warn line and substitutes `T::default()`, so an empty table gets
//!   served as real data: no error, no failing health check, smoke test
//!   passes. One unrecognised enum value in `item_table` blanks every item
//!   name and icon on a server.
//! - A refused service-account token: the jobs on that server stop quietly,
//!   and the fix needs a human.
//!
//! Optional: with `DISCORD_ALERT_WEBHOOK` unset this module does nothing.

use serde_json::json;

/// Env var holding a Discord webhook URL. Unset disables alerting entirely.
const WEBHOOK_ENV: &str = "DISCORD_ALERT_WEBHOOK";

/// Post the degraded-table list for one server, if there is anything to say.
pub async fn report_degraded_tables(client: &reqwest::Client, server: &str, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    tracing::warn!(
        server,
        count = warnings.len(),
        "game data loaded with degraded tables"
    );

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

    post(
        client,
        json!({
            "title": format!("Degraded game-data tables ({server})"),
            "description": description,
            "color": 0x00E0_A81A_u32,
            "footer": { "text": "These tables fell back to empty defaults. Data served from them is wrong, not missing." }
        }),
    )
    .await;
}

/// A service account's durable token was refused, so every job on that server
/// is stalled until a human logs it in again, hence the ping.
pub async fn report_service_account_refused(client: &reqwest::Client, server: &str, error: &str) {
    let login = if server == "bili" {
        "cargo run --bin set-session -- --server bili --bili-user <user> --bili-pass <pass>\n\
         # code 200007 (risk control): wait a few days and retry, or use another account"
    } else {
        "cargo run --bin set-session -- --server <server> --email <addr>\n\
         cargo run --bin set-session -- --server <server> --email <addr> --code <code>"
    }
    .replace("<server>", server);

    post(
        client,
        json!({
            "title": format!("Service account login refused ({server})"),
            "description": format!(
                "The stored `{server}` session no longer refreshes, so its pool details, \
                 event shops and probes are stalled.\n\n\
                 In `backend/`:\n```\n{login}\n```\n\
                 Then copy `game_sessions/{server}.json` to the VPS's `backend/game_sessions/`. \
                 The backend picks it up on its next attempt; no restart needed.\n\n\
                 Upstream said: `{}`",
                error.replace('`', "'")
            ),
            "color": 0x00D0_3A3A_u32,
        }),
    )
    .await;
}

/// A Trevor alert or its "recovered" follow-up (contract v1.1 section 2.2):
/// red when it fires, green when it clears. The caller decides when (one per
/// kind per cooldown); this only formats and posts.
pub async fn report_trevor(
    client: &reqwest::Client,
    kind: &str,
    title: &str,
    description: &str,
    recovered: bool,
) {
    if recovered {
        tracing::info!(kind, "trevor alert recovered");
    } else {
        tracing::warn!(kind, description, "trevor alert");
    }
    post(client, trevor_embed(kind, title, description, recovered)).await;
}

/// The embed of [`report_trevor`], pure so its shape is testable.
pub fn trevor_embed(
    kind: &str,
    title: &str,
    description: &str,
    recovered: bool,
) -> serde_json::Value {
    json!({
        "title": if recovered { format!("Trevor recovered: {title}") } else { format!("Trevor: {title}") },
        "description": description.replace('`', "'"),
        "color": if recovered { 0x002E_A043_u32 } else { 0x00D0_3A3A_u32 },
        "footer": { "text": format!("trevor alert `{kind}`") },
    })
}

/// Fire-and-forget: a failed webhook is logged and swallowed. Alerting must
/// never be able to take down the caller.
async fn post(client: &reqwest::Client, embed: serde_json::Value) {
    let Ok(webhook) = std::env::var(WEBHOOK_ENV) else {
        return;
    };
    if webhook.trim().is_empty() {
        return;
    }

    match client
        .post(&webhook)
        .json(&json!({ "embeds": [embed] }))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => tracing::error!(status = %resp.status(), "discord alert rejected"),
        Err(e) => tracing::error!(error = %e, "discord alert failed to send"),
    }
}
