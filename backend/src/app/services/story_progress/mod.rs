//! Story reading progress, synced to the account.
//!
//! The reader's progress is a localStorage document first and a row here
//! second: reading never waits on this table, and a browser that cannot reach
//! it keeps working unchanged. What the account buys is that the document
//! FOLLOWS the reader to another device.
//!
//! The document is stored verbatim. Its shape is owned by
//! `frontend/src/lib/story/progress.ts`, which coerces every field on read, and
//! the merge that decides what a round trip produces is
//! `frontend/src/lib/story/sync.ts`, which runs in the browser. Nothing here
//! merges: a PUT replaces the row whole.
//!
//! What this module does read is an ADMISSION check, not a parse. It answers
//! one question, "is this a story-progress document at all", so the column
//! cannot become a general blob store on an authenticated endpoint. Anything it
//! admits, it stores byte for byte.
//!
//! The account's OWN game data is a second source, and the two halves that read
//! it sit next door:
//!
//! - [`verdict`]: what the game says was read, out of a raw `syncData` payload.
//!   Pure: it takes JSON and the story index's maps and returns a set.
//!   `docs/story-reader.md` carries the census the rule was derived from.
//! - [`store`]: the database half, plus the re-derivation that runs the verdict
//!   rule again over what is already stored, with no call to the game server.

mod store;
mod verdict;

pub use store::{
    StoreFailure, StoreOutcome, StoredRead, reverdict, store_game_read, store_game_read_on,
    stored_payload,
};
pub use verdict::{GameStoryRead, GameStoryReadSet, parse_game_story_read, stage_satisfies};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use uuid::Uuid;

use crate::app::{error::ApiError, state::AppState};
use store::game_read_for;

/// The largest document accepted, in bytes of compact JSON.
///
/// 512 KB is roughly 4,000 stories carrying a saved position each, against a
/// corpus of 1,797 readable EN stories, so it is not a limit a reader reaches
/// by reading. It is the ceiling that keeps the column bounded.
pub const MAX_DOCUMENT_BYTES: usize = 512 * 1024;

/// The reading-progress document, exactly as the browser holds it.
///
/// Opaque on purpose: typing it here would put the format in two places and
/// make every reader-side field addition a backend migration. The TS type is
/// declared inline so the generated binding still names the real shape.
#[derive(Debug, Clone, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(transparent)]
#[schema(value_type = Object)]
#[ts(export)]
pub struct StoryProgressDocument(
    #[ts(
        type = "{ v: 1 | 2; read: Record<string, number>; unread?: Record<string, number>; pos: Record<string, { halt: number; total: number; ts: number; choices: Record<number, string> }>; last?: string }"
    )]
    pub Value,
);

/// What `GET /user/story-progress` answers.
#[derive(Debug, Clone, Serialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoryProgressResponse {
    /// The stored document, or null when this account has never synced.
    pub progress: Option<StoryProgressDocument>,
    /// Unix seconds of the last write, null alongside a null `progress`.
    #[ts(type = "number | null")]
    pub updated_at: Option<i64>,
    /// Story ids the GAME says this account has read, imported on the last
    /// game-data refresh. Empty when the account has never refreshed, or when
    /// the payload carried no story block.
    pub game_read: Vec<String>,
    /// Story ids the game's Archive lists that NOTHING played: unlocked by an
    /// event, never opened. A v1 document may carry these as read marks,
    /// because the first import baked its whole union into the document; the
    /// client's v1 -> v2 upgrade withdraws exactly these and nothing else.
    pub game_unread: Vec<String>,
    /// How many of `game_read` the game's ARCHIVE lists. The rest are known
    /// only from the client's played-script flags, which is the half that
    /// covers the mainline.
    pub game_archived: u32,
    /// Unix seconds of that import, null when nothing has ever been imported.
    #[ts(type = "number | null")]
    pub game_synced_at: Option<i64>,
}

/// A number that is finite and not negative.
///
/// `serde_json` already refuses NaN and infinity at parse time, so the finite
/// check is belt over braces; the sign is the one that bites, because a halt of
/// -1 is the reader's own "no position" sentinel and must not travel. One rule,
/// applied to every number the document carries: the `pos` entries' three
/// fields and the `unread` values.
fn non_negative(v: &Value) -> bool {
    v.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
}

/// The same check on a named field of an object, absent counting as missing.
fn field_non_negative(obj: &serde_json::Map<String, Value>, key: &str) -> bool {
    obj.get(key).is_some_and(non_negative)
}

/// Admit a document, or say why not.
///
/// Every rejection is a 400 naming the field: a client that gets one has a bug,
/// and a message that only says "invalid" costs a debugging session.
pub fn validate(doc: &StoryProgressDocument) -> Result<(), ApiError> {
    let Some(root) = doc.0.as_object() else {
        return Err(ApiError::BadRequest("progress must be an object".into()));
    };
    // v1 documents baked the game's read marks into `read`; v2 keeps `read`
    // to the reader's own marks and takes the game's from the response. Both
    // are stored as sent: the client upgrades, the server only checks shape.
    if !matches!(root.get("v").and_then(Value::as_i64), Some(1 | 2)) {
        return Err(ApiError::BadRequest("progress.v must be 1 or 2".into()));
    }
    let Some(read) = root.get("read").and_then(Value::as_object) else {
        return Err(ApiError::BadRequest(
            "progress.read must be an object".into(),
        ));
    };
    if read.keys().any(String::is_empty) {
        return Err(ApiError::BadRequest(
            "progress.read has an empty story id".into(),
        ));
    }
    // `unread` is optional: the reader's own overrides of the game's verdict,
    // story id -> epoch ms of the clear.
    if let Some(unread) = root.get("unread") {
        let Some(unread) = unread.as_object() else {
            return Err(ApiError::BadRequest(
                "progress.unread must be an object".into(),
            ));
        };
        if unread.keys().any(String::is_empty) {
            return Err(ApiError::BadRequest(
                "progress.unread has an empty story id".into(),
            ));
        }
        if !unread.values().all(non_negative) {
            return Err(ApiError::BadRequest(
                "progress.unread values must be finite numbers at or above zero".into(),
            ));
        }
    }
    let Some(pos) = root.get("pos").and_then(Value::as_object) else {
        return Err(ApiError::BadRequest(
            "progress.pos must be an object".into(),
        ));
    };
    for (id, entry) in pos {
        if id.is_empty() {
            return Err(ApiError::BadRequest(
                "progress.pos has an empty story id".into(),
            ));
        }
        let Some(entry) = entry.as_object() else {
            return Err(ApiError::BadRequest(format!(
                "progress.pos.{id} must be an object"
            )));
        };
        for key in ["halt", "total", "ts"] {
            if !field_non_negative(entry, key) {
                return Err(ApiError::BadRequest(format!(
                    "progress.pos.{id}.{key} must be a finite number at or above zero"
                )));
            }
        }
    }
    if let Some(last) = root.get("last")
        && !last.is_string()
    {
        return Err(ApiError::BadRequest(
            "progress.last must be a string".into(),
        ));
    }
    // Measured on the compact encoding, which is what the column stores, not on
    // the request body: whitespace a client happens to send is not the user's.
    let bytes = serde_json::to_vec(&doc.0)
        .map_err(|e| ApiError::BadRequest(format!("progress is not serializable: {e}")))?
        .len();
    if bytes > MAX_DOCUMENT_BYTES {
        return Err(ApiError::BadRequest(format!(
            "progress is {bytes} bytes, over the {MAX_DOCUMENT_BYTES} byte limit"
        )));
    }
    Ok(())
}

/// The stored document for one account, or an empty response when there is none.
pub async fn get(state: &AppState, user_id: Uuid) -> Result<StoryProgressResponse, ApiError> {
    let row: Option<(Value, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as("SELECT progress, updated_at FROM user_story_progress WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?;
    let (game_read, game_unread, game_archived, game_synced_at) =
        game_read_for(state, user_id).await?;
    let game_archived = u32::try_from(game_archived).unwrap_or(u32::MAX);
    Ok(match row {
        Some((progress, updated_at)) => StoryProgressResponse {
            progress: Some(StoryProgressDocument(progress)),
            updated_at: Some(updated_at.timestamp()),
            game_read,
            game_unread,
            game_archived,
            game_synced_at,
        },
        None => StoryProgressResponse {
            progress: None,
            updated_at: None,
            game_read,
            game_unread,
            game_archived,
            game_synced_at,
        },
    })
}

/// Replace the stored document whole, and answer with it as stored.
pub async fn put(
    state: &AppState,
    user_id: Uuid,
    doc: StoryProgressDocument,
) -> Result<StoryProgressResponse, ApiError> {
    validate(&doc)?;
    let (progress, updated_at): (Value, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        r"
        INSERT INTO user_story_progress (user_id, progress, updated_at)
        VALUES ($1, $2, now())
        ON CONFLICT (user_id) DO UPDATE SET progress = EXCLUDED.progress, updated_at = now()
        RETURNING progress, updated_at
        ",
    )
    .bind(user_id)
    .bind(&doc.0)
    .fetch_one(&state.db)
    .await?;
    // The write itself is unchanged: the document is replaced whole and nothing
    // reads it. The two game fields ride along because the response type is
    // shared with the GET, and answering them empty here would be a lie about
    // an account that has imported marks.
    let (game_read, game_unread, game_archived, game_synced_at) =
        game_read_for(state, user_id).await?;
    Ok(StoryProgressResponse {
        progress: Some(StoryProgressDocument(progress)),
        updated_at: Some(updated_at.timestamp()),
        game_read,
        game_unread,
        game_archived: u32::try_from(game_archived).unwrap_or(u32::MAX),
        game_synced_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(json: Value) -> StoryProgressDocument {
        StoryProgressDocument(json)
    }

    #[test]
    fn admits_an_empty_document() {
        assert!(validate(&doc(serde_json::json!({ "v": 1, "read": {}, "pos": {} }))).is_ok());
    }

    #[test]
    fn admits_a_full_document() {
        let d = doc(serde_json::json!({
            "v": 1,
            "read": { "main_00-01_beg": 1 },
            "pos": { "main_00-01_end": { "halt": 12, "total": 40, "ts": 1_759_000_000_000u64, "choices": { "0": "1" } } },
            "last": "main_00-01_end",
        }));
        assert!(validate(&d).is_ok());
    }

    #[test]
    fn refuses_a_wrong_version() {
        assert!(validate(&doc(serde_json::json!({ "v": 3, "read": {}, "pos": {} }))).is_err());
        assert!(validate(&doc(serde_json::json!({ "v": 2, "read": {}, "pos": {} }))).is_ok());
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 2, "read": {}, "pos": {}, "unread": { "a": 5 } })
            ))
            .is_ok()
        );
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 2, "read": {}, "pos": {}, "unread": { "": 5 } })
            ))
            .is_err()
        );
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 2, "read": {}, "pos": {}, "unread": [] })
            ))
            .is_err()
        );
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 2, "read": {}, "pos": {}, "unread": { "a": "x" } })
            ))
            .is_err()
        );
        assert!(validate(&doc(serde_json::json!({ "read": {}, "pos": {} }))).is_err());
    }

    #[test]
    fn refuses_a_non_object_root_or_branch() {
        assert!(validate(&doc(serde_json::json!([]))).is_err());
        assert!(validate(&doc(serde_json::json!({ "v": 1, "read": [], "pos": {} }))).is_err());
        assert!(validate(&doc(serde_json::json!({ "v": 1, "read": {}, "pos": 3 }))).is_err());
    }

    #[test]
    fn refuses_a_negative_or_missing_number() {
        let neg = serde_json::json!({ "v": 1, "read": {}, "pos": { "s": { "halt": -1, "total": 4, "ts": 1 } } });
        assert!(validate(&doc(neg)).is_err());
        let missing =
            serde_json::json!({ "v": 1, "read": {}, "pos": { "s": { "halt": 0, "ts": 1 } } });
        assert!(validate(&doc(missing)).is_err());
    }

    #[test]
    fn refuses_an_empty_story_id_and_a_non_string_last() {
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 1, "read": { "": 1 }, "pos": {} })
            ))
            .is_err()
        );
        assert!(
            validate(&doc(
                serde_json::json!({ "v": 1, "read": {}, "pos": {}, "last": 4 })
            ))
            .is_err()
        );
    }

    #[test]
    fn refuses_a_document_over_the_cap() {
        let mut read = serde_json::Map::new();
        for i in 0..40_000 {
            read.insert(format!("story_{i}"), serde_json::json!(1));
        }
        let big = doc(serde_json::json!({ "v": 1, "read": read, "pos": {} }));
        let err = validate(&big).unwrap_err();
        assert!(
            matches!(err, ApiError::BadRequest(ref m) if m.contains("byte limit")),
            "{err:?}"
        );
    }
}
