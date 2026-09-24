--
-- The read verdict, and the stage rule behind it.
--
-- v027 stored the union of the played-script flags and the Archive block and
-- served every row as read. Measured 2026-09-24 against the same account and
-- `story_review_table`'s own `RequiredStages`, that was wrong both ways: the
-- Archive lists what an event UNLOCKED (94 of its gated rows had never been
-- opened), and the flags never carry the 42 mainline story-only stages the
-- game plays as a stage. `cleared` records the third source, the account's
-- `dungeon.stages` read against the gates, and `read_in_game` is the verdict
-- the library receives: played, or cleared, or Archive-listed on a story with
-- no gate. Rows the verdict rejects stay for the census and are never served.
--
-- Existing rows were all served as read and stay that way until the account's
-- next refresh rewrites them under the new rule.
--
ALTER TABLE user_game_story_read
    ADD COLUMN IF NOT EXISTS cleared      boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS read_in_game boolean NOT NULL DEFAULT false;
UPDATE user_game_story_read SET read_in_game = true;
