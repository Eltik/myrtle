--
-- Reading progress for the story reader, one row per myrtle account.
--
-- The document is stored VERBATIM and never interpreted here: its shape is
-- owned by `frontend/src/lib/story/progress.ts`, which coerces every field on
-- read, so a hand-edited or older document degrades to "no progress" in the
-- browser rather than failing a query. The backend's only reading of it is the
-- admission check in `app/services/story_progress.rs` (version, key and number
-- shape, 512 KB cap), which exists to keep the column from becoming a general
-- blob store, not to define the format.
--
-- One row per user, replaced whole. The client merges local against remote
-- before it writes, so there is nothing to reconcile server-side and no history
-- worth keeping: `updated_at` is the only ordering this table needs.
--
CREATE TABLE IF NOT EXISTS user_story_progress (
    user_id    uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    progress   jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
