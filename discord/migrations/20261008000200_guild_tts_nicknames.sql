-- How a member's name is read out by text-to-speech, per server.
--
-- Every spoken message starts with "<name> said:"; a row here replaces the member's display name
-- in that prefix, for names the voice gets wrong. Set by the member themself with
-- `/tts nickname set`, or by a moderator (Manage Nicknames, or the mod role) for someone else;
-- set_by records which. Clearing deletes the row, so no row means the display name is used.
CREATE TABLE IF NOT EXISTS guild_tts_nicknames (
    guild_id INTEGER NOT NULL,
    user_id  INTEGER NOT NULL,
    nickname TEXT    NOT NULL,
    set_by   INTEGER NOT NULL,
    set_at   INTEGER NOT NULL,
    PRIMARY KEY (guild_id, user_id)
);
