--
-- Where players STOP investing: the mastery a skill is left at, and the level a
-- module is left at. Refreshed alongside the other build-stat aggregates by
-- core/operator_ownership_job.rs, and gated on user_settings.share_stats the
-- same way.
--
-- Both histograms are taken over E2 owners, and both use 0 for "not started".
-- Measured 2026-09-10, which is what fixed the denominators:
--
--   * Mastery rows exist for every owned operator, including 664,176 at E0 and
--     183,465 at E1 that are all specialize_level = 0 because mastery needs E2.
--     Counting them would drown every real figure in operators that CANNOT be
--     mastered, so the aggregate filters uo.elite = 2. Every one of the 154,965
--     mastered rows is already at E2, so the filter loses nothing real.
--
--   * user_operator_modules rows likewise exist below E2 (327,775 at E0, 82,392
--     at E1), and `locked` is the discriminator, not the row's presence: a
--     locked row ALWAYS carries module_level = 1, which is a placeholder and not
--     a level. Unlocked rows exist only at E2. Level 0 here therefore means
--     "has not unlocked it", remapped from locked = true, and 1..3 mean an
--     actually unlocked module at that level.
--
-- The shape of the answer, before any UI reads it: players do not stop in the
-- middle. M3 (131,383 rows) outnumbers M1 (13,906) and M2 (9,676) by roughly ten
-- to one, and module Lv3 (52,309) outnumbers Lv2 (9,901) by five to one. M1, M2
-- and Lv2 are states people pass through rather than settle at.
--

CREATE TABLE public.operator_mastery_stats (
    server_id smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    -- 0-based, matching user_operators.default_skill and
    -- user_operator_skills.skill_index. Resolved to a stable skill_id on the
    -- read path; never used as an array position by a client.
    skill_index smallint NOT NULL,
    -- 0 = E2 but no mastery, 1..3 = M1..M3.
    mastery smallint NOT NULL,
    users integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (server_id, operator_id, skill_index, mastery)
);

CREATE TABLE public.operator_module_level_stats (
    server_id smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    uni_equip_id character varying(50) NOT NULL,
    -- 0 = owns the operator at E2 but has not unlocked this module,
    -- 1..3 = unlocked at that level.
    module_level smallint NOT NULL,
    users integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (server_id, operator_id, uni_equip_id, module_level)
);

-- The detail page reads one operator across servers, mirroring idx_oscs_operator.
CREATE INDEX idx_oms_operator ON public.operator_mastery_stats USING btree (operator_id);
CREATE INDEX idx_omls_operator ON public.operator_module_level_stats USING btree (operator_id);
