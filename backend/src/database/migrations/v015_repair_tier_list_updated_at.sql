-- Repair fallout from v014.
--
-- v014's backfill issued `UPDATE tier_lists SET updated_at = <real activity>`,
-- but the pre-existing v003 BEFORE UPDATE trigger (fn_update_timestamp)
-- unconditionally forces NEW.updated_at = NOW() on every UPDATE. Result:
-- every tier list whose child activity was newer than its own updated_at got
-- stamped with v014's transaction timestamp instead of the intended
-- child-activity time — the UI now shows "just now updated" for those rows.
--
-- Fix:
--   1. Relax fn_update_timestamp so it only auto-bumps when the caller did
--      NOT explicitly set updated_at. Explicit SETs win; callers that forget
--      still get the safety net.
--   2. Re-run the backfill, scoped strictly to rows whose updated_at exactly
--      matches v014's applied_at — those are the rows v014 clobbered, and
--      only those. NOW() / transaction_timestamp() is constant within a
--      transaction, so the equality match is exact, not heuristic.

CREATE OR REPLACE FUNCTION fn_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at THEN
        NEW.updated_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

WITH v014_time AS (
    SELECT applied_at
    FROM _migrations
    WHERE name = 'v014_propagate_tier_list_updated_at'
)
UPDATE tier_lists tl
SET updated_at = sub.latest
FROM (
    SELECT tl.id,
           GREATEST(
               tl.created_at,
               COALESCE(MAX(tp.updated_at), tl.created_at),
               COALESCE(MAX(tlv.published_at), tl.created_at)
           ) AS latest
    FROM tier_lists tl
    LEFT JOIN tiers t ON t.tier_list_id = tl.id
    LEFT JOIN tier_placements tp ON tp.tier_id = t.id
    LEFT JOIN tier_list_versions tlv ON tlv.tier_list_id = tl.id
    GROUP BY tl.id, tl.created_at
) sub
WHERE tl.id = sub.id
  AND tl.updated_at = (SELECT applied_at FROM v014_time)
  AND sub.latest < tl.updated_at;
