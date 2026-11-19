//! Discord alerts for failures nothing else surfaces; with no webhook configured they only log.

use serde_json::{Value, json};

const MAX_LISTED: usize = 15; // Discord caps an embed description at 4096 characters

/// `None` when nothing is degraded
#[must_use]
pub fn degraded_tables_embed(server: &str, warnings: &[String]) -> Option<Value> {
    if warnings.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = warnings
        .iter()
        .take(MAX_LISTED)
        .map(|w| format!("• `{}`", w.replace('`', "'")))
        .collect();
    if warnings.len() > MAX_LISTED {
        lines.push(format!("…and {} more", warnings.len() - MAX_LISTED));
    }
    Some(json!({
        "title": format!("Degraded game-data tables ({server})"),
        "description": lines.join("\n"),
        "color": 0x00E0_A81A_u32,
        "footer": { "text": "These tables fell back to empty defaults. Data served from them is wrong, not missing." }
    }))
}

pub async fn report_degraded_tables(
    client: &reqwest::Client,
    webhook: Option<&str>,
    server: &str,
    warnings: &[String],
) {
    let Some(embed) = degraded_tables_embed(server, warnings) else {
        return;
    };
    tracing::warn!(
        server,
        count = warnings.len(),
        "game data loaded with degraded tables"
    );
    post(client, webhook, embed).await;
}

/// A failed post is logged and swallowed, so alerting never fails its caller
pub async fn post(client: &reqwest::Client, webhook: Option<&str>, embed: Value) {
    let Some(webhook) = webhook.filter(|w| !w.trim().is_empty()) else {
        return;
    };
    match client
        .post(webhook)
        .json(&json!({ "embeds": [embed] }))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {}
        Ok(resp) => tracing::error!(status = %resp.status(), "discord alert rejected"),
        Err(e) => tracing::error!(error = %e, "discord alert failed to send"),
    }
}
