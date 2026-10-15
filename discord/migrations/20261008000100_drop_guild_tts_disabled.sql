-- Drop the per-guild TTS off switch.
--
-- TTS no longer joins voice on its own: the bot reads a channel only after someone runs
-- `/tts join` there, so there is nothing left for a server to opt out of, and `/tts enable`,
-- `/tts disable` and `/tts status` are gone. The table is dropped here, not by deleting or editing
-- 20261008000000_guild_tts_disabled.sql: that migration is already applied, and sqlx refuses to
-- start when an applied migration's file is missing or its checksum changes.
DROP TABLE IF EXISTS guild_tts_disabled;
