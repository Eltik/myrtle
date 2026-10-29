-- Keep RA topics a sync does not carry.
--
-- Since the 2026-10-28 EN update, syncData's `sandboxPerm` holds only the topic
-- the player has loaded in game (`sandboxPerm.topic`); every other topic comes
-- back as `null` under `template`, with only a two-field `summary`
-- (`{"day": 42, "inChallenge": false}`). The sync procedures write the blob
-- with `ON CONFLICT DO UPDATE SET progress = EXCLUDED.progress`, so a player
-- who opened `sandbox_2` had their stored `sandbox_1` replaced by `null` and
-- its RA grade fell to 0. Measured on an EN dump of 2026-10-29: `sandbox_1` a
-- 21-key object on 2026-09-20, `null` on 2026-10-29, `topic` = `sandbox_2`.
--
-- On update, every `template.<key>.<topic>` the stored row holds as an object
-- and the new row does not is copied back. A topic the game sends as an object
-- (including a reset one) still replaces the stored one. A trigger, not a
-- procedure edit, so every writer is covered: each `sync_*` version and the
-- `user-data` import.

CREATE OR REPLACE FUNCTION keep_unloaded_sandbox_topics() RETURNS trigger AS $$
DECLARE
    tpl text;
    topic text;
    kept jsonb;
BEGIN
    IF jsonb_typeof(OLD.progress) IS DISTINCT FROM 'object'
        OR jsonb_typeof(NEW.progress) IS DISTINCT FROM 'object' THEN
        RETURN NEW;
    END IF;
    FOR tpl, topic, kept IN
        SELECT t.key, tp.key, tp.value
        FROM jsonb_each(
                CASE WHEN jsonb_typeof(OLD.progress -> 'template') = 'object'
                     THEN OLD.progress -> 'template' ELSE '{}'::jsonb END) t,
             jsonb_each(
                CASE WHEN jsonb_typeof(t.value) = 'object'
                     THEN t.value ELSE '{}'::jsonb END) tp
        WHERE jsonb_typeof(tp.value) = 'object'
    LOOP
        IF jsonb_typeof(NEW.progress #> ARRAY['template', tpl, topic]) IS DISTINCT FROM 'object' THEN
            IF jsonb_typeof(NEW.progress -> 'template') IS DISTINCT FROM 'object' THEN
                NEW.progress := jsonb_set(NEW.progress, '{template}', '{}'::jsonb);
            END IF;
            IF jsonb_typeof(NEW.progress -> 'template' -> tpl) IS DISTINCT FROM 'object' THEN
                NEW.progress := jsonb_set(NEW.progress, ARRAY['template', tpl], '{}'::jsonb);
            END IF;
            NEW.progress := jsonb_set(NEW.progress, ARRAY['template', tpl, topic], kept);
        END IF;
    END LOOP;
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS user_sandbox_progress_keep_unloaded ON user_sandbox_progress;
CREATE TRIGGER user_sandbox_progress_keep_unloaded
    BEFORE UPDATE ON user_sandbox_progress
    FOR EACH ROW EXECUTE FUNCTION keep_unloaded_sandbox_topics();
