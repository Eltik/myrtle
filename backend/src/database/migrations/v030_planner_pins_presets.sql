--
-- Operator planner: pinned groups and saved plan presets.
--
-- `plan_groups.pinned` floats a group to the top of the caller's group list.
-- Adding the column with a constant default rewrites no row, so the group
-- update trigger does not fire and every `updated_at` is unchanged.
--
-- `plan_presets` holds named, rarity-agnostic targets a bulk-add applies to
-- many operators at once. `target` is the preset body as JSON; its shape and
-- ranges are validated by the backend (`PresetTarget`), not here, so a new
-- optional field needs no migration.
--
ALTER TABLE plan_groups ADD COLUMN pinned boolean NOT NULL DEFAULT false;

CREATE TABLE plan_presets (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name varchar(100) NOT NULL,
    target jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT plan_presets_user_id_name_key UNIQUE (user_id, name)
);

CREATE TRIGGER trg_plan_presets_timestamp
    BEFORE UPDATE ON plan_presets
    FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();
