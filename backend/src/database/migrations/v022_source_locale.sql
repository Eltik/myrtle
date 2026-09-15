--
-- Which locale the source text is written in.
--
-- `en` was the source locale by convention: the string hardcoded in the
-- manifest, in the frontend's DEFAULT_LOCALE, and assumed by the progress
-- query. Convention is the wrong mechanism for it, because it made English
-- report 0% translated forever - there are no `ui_messages` rows for the
-- source locale, since its text lives in `ui_message_keys.source_text`. The
-- flag makes "this locale is complete by definition" something the query can
-- actually express, and makes the default visible and changeable rather than
-- compiled in.
--
ALTER TABLE locales ADD COLUMN IF NOT EXISTS is_source boolean NOT NULL DEFAULT false;

UPDATE locales SET is_source = true WHERE code = 'en';

-- Exactly one source locale. A second would make "complete by definition"
-- ambiguous and would give two different answers for the default.
CREATE UNIQUE INDEX IF NOT EXISTS idx_locales_single_source ON locales((is_source)) WHERE is_source;

-- The source locale is always enabled: it is the floor of every fallback
-- chain and the language the bundled catalog is written in.
UPDATE locales SET enabled = true WHERE is_source;
