--
-- What the GAME says the account has read, imported on every game-data refresh.
--
-- This is the second source of reading progress and it is the account's, not
-- the browser's. `user_story_progress` holds what was read HERE, in the site's
-- own reader, and is written only by the client; this table holds what was read
-- in the Arknights client itself, and is written only by a refresh. The two
-- never touch: the union of them is computed in the browser
-- (`frontend/src/lib/story/sync.ts`), which is the only place that holds both
-- plus the local document.
--
-- One row per story with a read count above zero. The block is expected to list
-- every story the Archives shows, read or not, and a row per unread story would
-- be roughly 1,887 rows per account carrying no information; `read_count` is
-- kept because a re-read count is information the block has and nothing else
-- does. `read_at` is the client's own timestamp for the read and is null where
-- the payload carries none.
--
-- A refresh whose payload carries NO story block leaves this table alone rather
-- than emptying it: a payload without the key says nothing about what was read.
--
CREATE TABLE IF NOT EXISTS user_game_story_read (
    user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    story_id   text NOT NULL,
    read_count int NOT NULL DEFAULT 0,
    read_at    timestamptz,
    synced_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, story_id)
);
