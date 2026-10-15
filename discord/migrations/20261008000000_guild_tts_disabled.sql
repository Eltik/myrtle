-- Guilds where text-to-speech in voice channels is switched off.
--
-- TTS is on in every guild by default, so this table stores the exception: a row means a
-- moderator ran `/tts disable`, and `/tts enable` deletes it. No row means enabled, which is why a
-- guild the bot has just joined needs no setup. disabled_at is the unix time of the switch, shown
-- by `/tts status`.
CREATE TABLE IF NOT EXISTS guild_tts_disabled (
    guild_id    INTEGER PRIMARY KEY NOT NULL,
    disabled_at INTEGER NOT NULL
);
