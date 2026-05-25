-- Propagate child-table mutations up to tier_lists.updated_at.
--
-- Without these triggers, editing tiers, moving/adding/removing placements,
-- or publishing a new version leaves tier_lists.updated_at stale, so the UI's
-- "updated X ago" label drifts away from the actual last-edit time. The
-- existing per-row triggers in v003 only refresh a row's own updated_at on
-- direct UPDATEs to that row; they do not cross table boundaries.

-- tiers -> tier_lists
CREATE OR REPLACE FUNCTION fn_bump_tier_list_from_tier()
RETURNS TRIGGER AS $$
DECLARE
    target_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        target_id := OLD.tier_list_id;
    ELSE
        target_id := NEW.tier_list_id;
    END IF;

    UPDATE tier_lists SET updated_at = NOW() WHERE id = target_id;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_tiers_bump_tier_list ON tiers;
CREATE TRIGGER trg_tiers_bump_tier_list
AFTER INSERT OR UPDATE OR DELETE ON tiers
FOR EACH ROW EXECUTE FUNCTION fn_bump_tier_list_from_tier();

-- tier_placements -> tier_lists (via tiers.tier_list_id)
CREATE OR REPLACE FUNCTION fn_bump_tier_list_from_placement()
RETURNS TRIGGER AS $$
DECLARE
    target_id UUID;
    lookup_tier UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        lookup_tier := OLD.tier_id;
    ELSE
        lookup_tier := NEW.tier_id;
    END IF;

    SELECT tier_list_id INTO target_id FROM tiers WHERE id = lookup_tier;

    IF target_id IS NOT NULL THEN
        UPDATE tier_lists SET updated_at = NOW() WHERE id = target_id;
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_tier_placements_bump_tier_list ON tier_placements;
CREATE TRIGGER trg_tier_placements_bump_tier_list
AFTER INSERT OR UPDATE OR DELETE ON tier_placements
FOR EACH ROW EXECUTE FUNCTION fn_bump_tier_list_from_placement();

-- tier_list_versions -> tier_lists (publish bumps the parent too)
CREATE OR REPLACE FUNCTION fn_bump_tier_list_from_version()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE tier_lists SET updated_at = NOW() WHERE id = NEW.tier_list_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_tier_list_versions_bump_tier_list ON tier_list_versions;
CREATE TRIGGER trg_tier_list_versions_bump_tier_list
AFTER INSERT ON tier_list_versions
FOR EACH ROW EXECUTE FUNCTION fn_bump_tier_list_from_version();

-- Backfill: align updated_at with the most recent observable activity so the
-- UI doesn't keep showing stale timestamps for tier lists edited before this
-- migration ran. Uses the greater of the current value, the latest placement
-- timestamp, and the latest published_at.
UPDATE tier_lists tl
SET updated_at = sub.latest
FROM (
    SELECT tl.id,
           GREATEST(
               tl.updated_at,
               COALESCE(MAX(tp.updated_at), tl.updated_at),
               COALESCE(MAX(tlv.published_at), tl.updated_at)
           ) AS latest
    FROM tier_lists tl
    LEFT JOIN tiers t ON t.tier_list_id = tl.id
    LEFT JOIN tier_placements tp ON tp.tier_id = t.id
    LEFT JOIN tier_list_versions tlv ON tlv.tier_list_id = tl.id
    GROUP BY tl.id, tl.updated_at
) sub
WHERE tl.id = sub.id
  AND sub.latest > tl.updated_at;
