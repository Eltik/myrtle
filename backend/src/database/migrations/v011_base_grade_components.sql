-- The base grade's two components, stored at grade time so the profile can
-- show WHICH term drags the score: stationing utilization (actual vs the
-- optimizer's best on the built rooms) and infrastructure completeness
-- (built rooms vs the same rooms at max level). Nullable: rows graded before
-- this migration carry no split until the next (re)grade.
ALTER TABLE user_scores
    ADD COLUMN IF NOT EXISTS base_utilization double precision,
    ADD COLUMN IF NOT EXISTS base_infrastructure double precision;
