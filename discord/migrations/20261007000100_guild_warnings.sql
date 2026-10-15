-- Moderator warnings.
--
-- One row per warning, never updated: removing a warning deletes its row. The id is
-- AUTOINCREMENT rather than a plain rowid so an id is never reused after a delete, and an id
-- quoted in an old mod-log message can't come to mean a different warning. Ids are global
-- across guilds; every query that takes one also filters on guild_id.
--
-- created_at is unix seconds, so the escalation window is a plain integer comparison.
CREATE TABLE IF NOT EXISTS guild_warnings (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id     INTEGER NOT NULL,
    user_id      INTEGER NOT NULL,
    moderator_id INTEGER NOT NULL,
    reason       TEXT    NOT NULL,
    created_at   INTEGER NOT NULL
);

-- Every read (list, count for escalation, clear) is "this user in this guild".
CREATE INDEX IF NOT EXISTS idx_guild_warnings_guild_user ON guild_warnings(guild_id, user_id);

-- Per-guild escalation: once a member holds `threshold` warnings, apply `action`.
--
-- No row means no escalation, which is the default, so warnings never punish anyone until an
-- admin asks for it. window_days NULL counts every warning ever given; otherwise only the last
-- N days count. timeout_secs is only read when action = 'timeout'.
CREATE TABLE IF NOT EXISTS guild_warn_policy (
    guild_id     INTEGER PRIMARY KEY NOT NULL,
    threshold    INTEGER NOT NULL,
    window_days  INTEGER,
    action       TEXT    NOT NULL, -- timeout | kick | ban
    timeout_secs INTEGER
);
