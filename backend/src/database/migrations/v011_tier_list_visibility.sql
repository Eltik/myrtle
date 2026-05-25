ALTER TABLE tier_lists
    ADD COLUMN IF NOT EXISTS is_listed BOOLEAN NOT NULL DEFAULT true;

CREATE INDEX IF NOT EXISTS idx_tier_lists_is_listed
    ON tier_lists (is_listed)
    WHERE is_active = true;
