--
-- What "read in game" means, corrected.
--
-- v026 stored one row per story whose Archive re-read count was above zero.
-- Measured against a real EN `account/syncData` payload, `rc` is 0 on 1,036 of
-- the 1,038 stories the Archive block lists: it counts RE-READS, not reads, so
-- that predicate imported two stories for an account that has read most of the
-- library. The block's own PRESENCE is the signal, and `uts` is the Archive
-- unlock time rather than a read time, so both columns are renamed to say what
-- they hold.
--
-- The second source is new here. `user.status.flags` carries 1,281 set flags,
-- 1,084 of which are exactly `story_review_table` `InfoUnlockDatas[].StoryTxt`
-- script paths: that is the client's own "this story has been played" record,
-- and it is the only one that covers the mainline, which the Archive block
-- omits entirely. `played` and `archived` say which source named the row; a row
-- exists when either did. The union for the measured account is 1,365 rows.
--
ALTER TABLE user_game_story_read RENAME COLUMN read_count TO reread_count;
ALTER TABLE user_game_story_read RENAME COLUMN read_at TO unlocked_at;
ALTER TABLE user_game_story_read
    ADD COLUMN IF NOT EXISTS played   boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS archived boolean NOT NULL DEFAULT false;
