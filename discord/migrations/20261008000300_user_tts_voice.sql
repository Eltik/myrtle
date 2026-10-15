-- The text-to-speech voice each member picked with `/tts voice set`.
--
-- Global per user, not per server: a voice is how someone wants to sound, which doesn't change
-- between servers. voice is a `tts::voices` id ("en-gb", "ja", ...). An id that later leaves the
-- voice list is kept as stored and read as the default voice, so removing a voice never breaks a
-- row. No row means the default voice (`tts.default_voice`).
CREATE TABLE IF NOT EXISTS user_tts_voice (
    user_id INTEGER PRIMARY KEY NOT NULL,
    voice   TEXT    NOT NULL,
    set_at  INTEGER NOT NULL
);
