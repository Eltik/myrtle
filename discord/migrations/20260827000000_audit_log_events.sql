-- Per-guild audit-log event filter.
--
-- Stores the DISABLED set rather than the enabled one: 0 means "nothing switched off", so
-- existing bindings keep mirroring everything exactly as they did before this column existed,
-- and a category added in a later release starts out logging for every guild instead of
-- silently staying off until each admin notices it.
ALTER TABLE guild_audit_log ADD COLUMN disabled_events INTEGER NOT NULL DEFAULT 0;
