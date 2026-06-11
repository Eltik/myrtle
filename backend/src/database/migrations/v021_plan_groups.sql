CREATE TABLE plan_groups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE TRIGGER trg_plan_groups_timestamp
BEFORE UPDATE ON plan_groups
FOR EACH ROW
EXECUTE FUNCTION fn_update_timestamp();

CREATE TABLE plan_group_members (
    plan_group_id    UUID NOT NULL REFERENCES plan_groups(id) ON DELETE CASCADE,
    operator_plan_id UUID NOT NULL REFERENCES operator_plans(id) ON DELETE CASCADE,
    PRIMARY KEY (plan_group_id, operator_plan_id)
);
