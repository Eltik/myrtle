-- Per-guild asset-announcement server filter.
--
-- A comma-separated list of the asset-pipeline labels a guild hears from ("EN,JP"). NULL means
-- every configured server, so rows bound before this column existed keep receiving exactly what
-- they did, and a server added to config.json later reaches them without anyone opting in.
-- Labels are stored as configured; the watcher compares them case-insensitively.
ALTER TABLE guild_asset_channel ADD COLUMN servers TEXT;
