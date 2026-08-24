-- Player-declared base "account facts" the game sync cannot read (e.g. recruit
-- slots purchased beyond the first). Stored per user so the interactive
-- planner and /user/improvements price the same skills the same way.
ALTER TABLE user_settings ADD COLUMN IF NOT EXISTS base_facts JSONB;
