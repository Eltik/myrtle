CREATE TABLE jobs (
    id               bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    kind             text        NOT NULL,
    server_id        smallint,
    payload          jsonb       NOT NULL DEFAULT '{}'::jsonb,
    state            text        NOT NULL DEFAULT 'queued'
        CHECK (state IN ('queued', 'running', 'succeeded', 'dead')),
    priority         smallint    NOT NULL DEFAULT 0,
    attempts         smallint    NOT NULL DEFAULT 0,
    max_attempts     smallint    NOT NULL CHECK (max_attempts > 0),
    lease_secs       integer     NOT NULL CHECK (lease_secs > 0),
    run_at           timestamptz NOT NULL DEFAULT now(),
    dedupe_key       text,
    leased_by        uuid,
    lease_expires_at timestamptz,
    last_error       text,
    output           jsonb,
    created_at       timestamptz NOT NULL DEFAULT now(),
    finished_at      timestamptz,
    CONSTRAINT jobs_lease_matches_state
        CHECK ((state = 'running') = (leased_by IS NOT NULL AND lease_expires_at IS NOT NULL)),
    CONSTRAINT jobs_finished_matches_state
        CHECK ((state IN ('succeeded', 'dead')) = (finished_at IS NOT NULL))
);

ALTER TABLE jobs SET (fillfactor = 70, autovacuum_vacuum_scale_factor = 0.02);

CREATE INDEX jobs_claimable_idx ON jobs (priority DESC, run_at, id) WHERE state = 'queued';

CREATE INDEX jobs_running_idx ON jobs (id) WHERE state = 'running';

CREATE UNIQUE INDEX jobs_dedupe_idx ON jobs (kind, dedupe_key) WHERE state = 'queued';

CREATE INDEX jobs_finished_idx ON jobs (finished_at) WHERE finished_at IS NOT NULL;

CREATE TABLE job_schedules (
    kind             text        NOT NULL,
    scope            text        NOT NULL DEFAULT '',
    every_secs       integer     NOT NULL CHECK (every_secs > 0),
    offset_secs      integer     NOT NULL DEFAULT 0 CHECK (offset_secs >= 0),
    enabled          boolean     NOT NULL DEFAULT true,
    next_run_at      timestamptz NOT NULL DEFAULT now(),
    last_enqueued_at timestamptz,
    memory           jsonb       NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (kind, scope)
);
