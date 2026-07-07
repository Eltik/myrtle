--
-- Per-server medal ownership aggregate, refreshed daily by the background job
-- (core/medal_ownership_job.rs). Mirrors operator_ownership_stats: owners and
-- population count only stat-sharing users, so percentages reflect the sharing
-- population rather than every registered account.
--

CREATE TABLE public.medal_ownership_stats (
    server_id smallint NOT NULL,
    medal_id character varying(50) NOT NULL,
    owners integer DEFAULT 0 NOT NULL,
    population integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (server_id, medal_id)
);
