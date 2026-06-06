CREATE TABLE operator_plans (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    operator_id         VARCHAR(50) NOT NULL,
    target_elite        SMALLINT NOT NULL DEFAULT 0,
    target_level        SMALLINT NOT NULL DEFAULT 1,
    target_skill_level  SMALLINT NOT NULL DEFAULT 1,
    target_skills       JSONB NOT NULL DEFAULT '[]'::JSONB,
    target_modules      JSONB NOT NULL DEFAULT '[]'::JSONB,
    display_on_profile  BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, operator_id)
);

CREATE TRIGGER trg_operator_plans_timestamp
BEFORE UPDATE ON operator_plans
FOR EACH ROW
EXECUTE FUNCTION fn_update_timestamp();
