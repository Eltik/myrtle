use std::sync::Arc;

use crate::api::gamedata::GameData;
use crate::types::{Context, Data, Error};

pub mod admin;
pub mod api;
pub mod assets;
pub mod auditlog;
pub mod birthday;
pub mod collection;
pub mod general;
pub mod operator;
pub mod tts;
pub mod voiceline;
pub mod warn;

#[must_use]
pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        general::ping(),
        general::say(),
        admin::embed(),
        admin::ban_user(),
        admin::unban_user(),
        admin::kick_user(),
        admin::purge(),
        admin::modrole(),
        admin::autorole(),
        admin::antispam(),
        admin::reactionrole(),
        api::api(),
        assets::assets(),
        auditlog::auditlog(),
        warn::warn(),
        collection::collection(),
        birthday::birthday(),
        tts::tts(),
        voiceline::voiceline(),
    ]
}

/// The error a command returns when a game-data list won't load.
///
/// The reason goes to the log (`<context>: <list> list: <reason>`); the user gets a plain
/// sentence with no URL or error chain, because [`crate::hooks::on_error`] shows a command's
/// error text as it is.
pub fn list_unavailable(context: &str, list: &str, e: &Error) -> Error {
    tracing::warn!("{context}: {list} list: {e}");
    format!("Couldn't load the {list} list right now. Try again in a minute.").into()
}

/// Shared start of every game-data lookup (`/collection`, `/birthday today` and `upcoming`).
///
/// Checks the backend is configured and, when a list still has to be fetched for the first
/// time, defers so the fetch can't time the interaction out.
///
/// Deferring is public, so a no-match after a cold start answers publicly too; once the lists
/// are cached (they are warmed at startup) every no-match is ephemeral.
pub async fn begin_lookup(ctx: Context<'_>) -> Result<Arc<GameData>, Error> {
    let gamedata = Arc::clone(&ctx.data().gamedata);
    gamedata.base()?;
    if !gamedata.is_warm().await {
        ctx.defer().await?;
    }
    Ok(gamedata)
}
