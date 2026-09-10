--
-- Community build rates: E2 conversion plus the per-operator default-skill and
-- default-module distributions. All three are refreshed alongside the existing
-- ownership aggregate by core/operator_ownership_job.rs, and all three count
-- only stat-sharing users, so percentages reflect the sharing population.
--
-- Measured against 619,706 user_operators rows on 2026-09-10, which decided the
-- shape here:
--
--   * user_operators.default_skill is 0-BASED (0 = S1) with -1 as a "no skill
--     exists" sentinel, so the aggregate filters default_skill >= 0. It records
--     player choice, not client auto-assignment: among 63,403 rows carrying
--     exactly one M3 skill, the default names that skill 95.57% of the time.
--   * user_operators.current_equip is NULL on 72.75% of rows and holds an
--     ORIGINAL (uniequip_001_*, Type_ = INITIAL) id on a further 13.62%. Both
--     mean "no module chosen", so only the remaining 13.63% count.
--   * The distributions are stored whole rather than reduced to a winner: 12 of
--     364 operators have a modal share under 50%, where the runner-up is the
--     more honest headline.
--

-- E2 conversion rides on the existing ownership row: same key, same refresh,
-- same cache prefix. `owners` is already there as the denominator.
ALTER TABLE public.operator_ownership_stats
    ADD COLUMN e2_owners integer DEFAULT 0 NOT NULL;

--
-- Default-skill distribution over E2 owners.
--
-- skill_index is the game's own 0-based position and is what user_operators
-- stores, so it is unavoidable as the key. Position as identity is what broke
-- module ordering (17 of 380 operators disagreed with the game), so the service
-- resolves skill_index to a stable skill_id from gamedata on the READ path and
-- the API returns the id; consumers never index into an array by position.
--
-- Ruled out: storing skill_id here as a column. Postgres has no gamedata, so it
-- would have to be back-filled by the job in a second pass, and because the
-- whole aggregate is recomputed every 60 seconds there is no history for a
-- compute-time snapshot to protect. That makes it a denormalized copy with a
-- staleness window across a gamedata update and no benefit over resolving fresh.
--
CREATE TABLE public.operator_skill_choice_stats (
    server_id smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    skill_index smallint NOT NULL,
    users integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (server_id, operator_id, skill_index)
);

--
-- Default-module distribution over owners with an ADVANCED module equipped.
-- uni_equip_id is already a stable identity, so this table needs no companion
-- column the way the skill table does.
--
CREATE TABLE public.operator_module_choice_stats (
    server_id smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    uni_equip_id character varying(50) NOT NULL,
    users integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (server_id, operator_id, uni_equip_id)
);

-- The detail page reads one operator across servers; the listing never reads
-- these tables at all. Mirrors idx_oos_operator on the ownership table.
CREATE INDEX idx_oscs_operator ON public.operator_skill_choice_stats USING btree (operator_id);
CREATE INDEX idx_omcs_operator ON public.operator_module_choice_stats USING btree (operator_id);
