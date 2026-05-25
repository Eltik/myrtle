-- Gacha batches commonly contain the same operator twice in a single 10-pull
-- (the API stamps every row in the batch with one shared pull_timestamp).
-- The old UNIQUE (user_id, pull_timestamp, char_id, pool_id) silently dropped
-- the second copy, so a 10-pull with two of the same operator got persisted as
-- 9 rows. Adding a positional discriminator within the batch fixes this.

ALTER TABLE gacha_records
    ADD COLUMN IF NOT EXISTS batch_index SMALLINT NOT NULL DEFAULT 0;

-- Backfill existing rows with sequential indexes within each batch, ordered by
-- the row's insertion id. Only batches Yostar no longer returns will keep
-- these values long-term — batches still in the API window get rebuilt on the
-- next sync (see sp_insert_gacha_batch below).
WITH numbered AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY user_id, pull_timestamp, pool_id
               ORDER BY id
           ) - 1 AS idx
    FROM gacha_records
)
UPDATE gacha_records gr
SET batch_index = n.idx
FROM numbered n
WHERE gr.id = n.id;

-- Replace the old UNIQUE constraint. Auto-named, but the name can vary across
-- environments — look it up by columns rather than guessing.
DO $$
DECLARE
    v_conname TEXT;
BEGIN
    SELECT c.conname INTO v_conname
    FROM pg_constraint c
    WHERE c.conrelid = 'gacha_records'::regclass
      AND c.contype = 'u'
      AND (
          SELECT array_agg(att.attname ORDER BY att.attname)
          FROM unnest(c.conkey) AS k(attnum)
          JOIN pg_attribute att
            ON att.attrelid = c.conrelid AND att.attnum = k.attnum
      ) = ARRAY['char_id', 'pool_id', 'pull_timestamp', 'user_id']::name[];

    IF v_conname IS NOT NULL THEN
        EXECUTE format('ALTER TABLE gacha_records DROP CONSTRAINT %I', v_conname);
    END IF;
END$$;

ALTER TABLE gacha_records
    ADD CONSTRAINT gacha_records_batch_unique
    UNIQUE (user_id, pull_timestamp, char_id, pool_id, batch_index);

-- Replace the insert procedure. The old version used ON CONFLICT DO NOTHING,
-- which can't recover from the dropped-duplicate bug — a re-sync would just
-- re-collide on the same key. Instead, for every (pull_timestamp, pool_id)
-- present in the incoming data, delete the existing rows and re-insert from
-- the API payload. Yostar is authoritative for any batch it still returns,
-- and the Rust caller (`fetch_and_store`) accumulates all pages before calling
-- this proc, so the incoming set is the complete batch from the API's side.
-- Older batches the API no longer returns are not touched.
CREATE OR REPLACE PROCEDURE sp_insert_gacha_batch(
    p_user_id UUID,
    p_records JSONB   -- [{char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type, batch_index}]
)
LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM gacha_records gr
    WHERE gr.user_id = p_user_id
      AND (gr.pull_timestamp, gr.pool_id) IN (
          SELECT DISTINCT (r->>'pull_timestamp')::BIGINT, r->>'pool_id'
          FROM jsonb_array_elements(p_records) AS r
      );

    INSERT INTO gacha_records (
        user_id, char_id, pool_id, rarity,
        pull_timestamp, pool_name, gacha_type, batch_index
    )
    SELECT
        p_user_id,
        r->>'char_id',
        r->>'pool_id',
        (r->>'rarity')::SMALLINT,
        (r->>'pull_timestamp')::BIGINT,
        r->>'pool_name',
        r->>'gacha_type',
        (r->>'batch_index')::SMALLINT
    FROM jsonb_array_elements(p_records) AS r;
END;
$$;
