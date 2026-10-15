-- Operator birthday announcements.
--
-- last_posted_date is the game day (YYYY-MM-DD, the UTC-7 date at the 04:00 UTC-7 reset) last
-- announced in this guild. The scheduler posts whenever it differs from the current game day,
-- so a restart can't double-post and a bot that was down at reset catches up on its next tick.
-- Binding a channel writes the current game day here, so a channel bound after today's reset
-- starts tomorrow instead of firing at once.
CREATE TABLE IF NOT EXISTS guild_birthday_channel (
    guild_id         INTEGER PRIMARY KEY NOT NULL,
    channel_id       INTEGER NOT NULL,
    last_posted_date TEXT
);
