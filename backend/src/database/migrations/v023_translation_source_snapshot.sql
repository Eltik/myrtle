--
-- The English a translation was actually written against.
--
-- `ui_messages.source_hash` already says THAT the English moved; it cannot say
-- WHAT moved, because `ui_message_keys.source_text` is overwritten in place by
-- every extractor sync. A translator opening a stale row therefore saw the new
-- English and a badge, and had to guess which clause changed - on a 4,000-key
-- catalogue that is the difference between a re-read and a re-translation.
--
-- Storing the source text next to the hash it belongs to keeps the comparison
-- local to the row: old English is `ui_messages.source_text`, new English is
-- `ui_message_keys.source_text`, and no history table has to be pruned.
--
ALTER TABLE ui_messages ADD COLUMN IF NOT EXISTS source_text text;

-- A row whose stamp still matches the key was translated against exactly the
-- English that is there now, so its snapshot is recoverable. A row that is
-- already stale is not: that English is gone, and NULL says so rather than
-- backfilling a text the translator never saw.
UPDATE ui_messages m
   SET source_text = k.source_text
  FROM ui_message_keys k
 WHERE k.key = m.key
   AND m.source_hash = k.source_hash
   AND m.source_text IS NULL;
