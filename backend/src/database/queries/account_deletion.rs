//! Self-serve account deletion: the signed-in user and everything that is
//! theirs, in the caller's transaction.
//!
//! Most of the work is `DELETE FROM users`, whose foreign keys decide each
//! table. What the keys cannot decide is done here, around that one statement.
//!
//! Deleted with the account (`ON DELETE CASCADE` from `users`): every `user_*`
//! table (`user_operator_skills` / `_modules` through `user_operators`, and
//! the saved and synced squads in `user_squads` and `user_game_squads`),
//! `gacha_records`, `operator_plans`, `plan_groups` (and `plan_group_members`
//! under both), `plan_presets`, `release_plans`, `grids`, `tier_list_favorites`,
//! `tier_list_permissions` held by the user, `translation_permissions` held by
//! the user, `ui_message_audit_log` rows the user wrote,
//! `leaderboard_snapshot_entries`, and the music library's `music_likes`,
//! `music_history` and `music_playlists` (with `music_playlist_tracks` under
//! it).
//!
//! Anonymized (`ON DELETE SET NULL`), because the row is shared content other
//! people keep reading: `tier_lists.created_by` on official lists,
//! `tier_list_versions.published_by`, `tier_list_view_events.user_id`,
//! `translation_permissions.granted_by`, `ui_messages.updated_by`,
//! `ui_documents.updated_by` and `gamedata_overrides.updated_by`.
//!
//! Done by hand below, before or after that statement:
//!
//! - Community tier lists the user created are DELETED, not anonymized. They
//!   are the user's own writing, and nothing of anyone else's hangs off one: a
//!   tier list has no forks, so what other people lose is a favourite, a view
//!   count or an edit grant on that list. Official lists are site content and
//!   keep the `SET NULL` above. A fork someone made of one of the user's grids
//!   survives: a fork is a full copy and `template_of` goes `NULL`.
//! - `tier_list_permissions.granted_by` (no action on delete, so it would block
//!   the delete) and the non-key columns `audit_log.changed_by` and
//!   `release_overrides.updated_by` are set `NULL`: the grant, the audit row and
//!   the override are someone else's and stay.
//! - `operator_notes_audit_log` rows the user wrote are deleted, the same choice
//!   the schema already made for `ui_message_audit_log`. `changed_by` is
//!   `NOT NULL` and the notes themselves stay; only the trail naming the editor
//!   goes.
//! - `audit_log` rows for `users` and `user_scores` keyed on the account are
//!   deleted AFTER the account, because the audit trigger on both tables writes
//!   a `DELETE` row holding the whole old row (nickname, uid, scores) as the
//!   account goes. The same holds for the deleted community lists and their
//!   placements.

use sqlx::PgConnection;
use uuid::Uuid;

/// What a deletion took, for cache invalidation and the log.
#[derive(Debug)]
pub struct DeletedAccount {
    /// The game uid the account was keyed on, for the `uid`-keyed caches.
    pub uid: String,
    /// Slugs of the community tier lists deleted with the account.
    pub tier_list_slugs: Vec<String>,
}

/// The by-hand statements the module docs list, each bound to the account id
/// as `$1`, run in this order before `DELETE FROM users`.
const BEFORE_DELETE: [&str; 4] = [
    "UPDATE tier_list_permissions SET granted_by = NULL WHERE granted_by = $1",
    "DELETE FROM operator_notes_audit_log WHERE changed_by = $1",
    "UPDATE audit_log SET changed_by = NULL WHERE changed_by = $1",
    "UPDATE release_overrides SET updated_by = NULL WHERE updated_by = $1",
];

/// Delete `user_id` and everything that is theirs. `None` when no such account
/// exists. Runs every statement on `conn`, so the caller owns the transaction:
/// the route commits it, the test rolls it back.
pub async fn delete_account(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Option<DeletedAccount>, sqlx::Error> {
    // Lock the row first, so two deletes of one account serialize and the
    // second finds nothing.
    let Some(uid) =
        sqlx::query_scalar::<_, String>("SELECT uid FROM users WHERE id = $1 FOR UPDATE")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await?
    else {
        return Ok(None);
    };

    let lists: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, slug FROM tier_lists WHERE created_by = $1 AND list_type = 'community'",
    )
    .bind(user_id)
    .fetch_all(&mut *conn)
    .await?;
    let list_ids: Vec<Uuid> = lists.iter().map(|(id, _)| *id).collect();
    let tier_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM tiers WHERE tier_list_id = ANY($1)")
            .bind(&list_ids)
            .fetch_all(&mut *conn)
            .await?;

    sqlx::query("DELETE FROM tier_lists WHERE id = ANY($1)")
        .bind(&list_ids)
        .execute(&mut *conn)
        .await?;
    for sql in BEFORE_DELETE {
        sqlx::query(sql).bind(user_id).execute(&mut *conn).await?;
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;

    // Last, so the rows the triggers wrote during the deletes above go too.
    sqlx::query(
        "DELETE FROM audit_log
         WHERE (table_name IN ('users', 'user_scores') AND record_id = $1::text)
            OR (table_name = 'tier_lists' AND record_id = ANY($2::uuid[]::text[]))
            OR (table_name = 'tier_placements'
                AND COALESCE(new_data, old_data)->>'tier_id' = ANY($3::uuid[]::text[]))",
    )
    .bind(user_id)
    .bind(&list_ids)
    .bind(&tier_ids)
    .execute(&mut *conn)
    .await?;

    Ok(Some(DeletedAccount {
        uid,
        tier_list_slugs: lists.into_iter().map(|(_, slug)| slug).collect(),
    }))
}
