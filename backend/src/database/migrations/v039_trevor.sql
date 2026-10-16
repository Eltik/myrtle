-- Trevor, the story question answerer (design: trevor/design/trevor-integration.md;
-- binding interface: trevor/design/trevor-contract-v1.md).
--
-- The VPS holds only plumbing: the published corpus versions and their panel
-- JSON, a job queue a worker pulls from, the answers it wrote back, the
-- workers' last heartbeats, a daily Cloudflare Neuron ledger and reader
-- feedback. Every row carries `server` ('en' now, 'cn' later), so a second
-- corpus is a second version stream, not a redesign.
--
-- Spoilers use one integer scale, the gatePos: a story's 1-based position in
-- Trevor's EN release order (contract section 1). A job's horizon is a
-- gatePos (`horizon_pos`), and answers are cached per era boundary
-- (`era_pos`), so every reader past the same boundary shares one answer.

CREATE TABLE trevor_versions (
    version      TEXT        NOT NULL,
    server       TEXT        NOT NULL,
    built_at     TIMESTAMPTZ,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    gate_report  JSONB,
    -- Carries `builtFrom` (the EN res_version), `storyOrder` (story ids in
    -- gatePos order) and `eras` ([{id, name, position}]).
    manifest     JSONB,
    active       BOOLEAN     NOT NULL DEFAULT false,
    PRIMARY KEY (version, server)
);
-- At most one active version per server; activation flips it in one transaction.
CREATE UNIQUE INDEX trevor_versions_one_active ON trevor_versions (server) WHERE active;

-- The deterministic per-story panels of one version (no model at read time).
-- `gate_pos` is the panel's `position`: the profile horizon reads it.
CREATE TABLE trevor_panels (
    version  TEXT    NOT NULL,
    server   TEXT    NOT NULL,
    story_id TEXT    NOT NULL,
    gate_pos INTEGER,
    panels   JSONB   NOT NULL,
    PRIMARY KEY (version, server, story_id),
    FOREIGN KEY (version, server) REFERENCES trevor_versions (version, server) ON DELETE CASCADE
);

-- `era_pos` is the era boundary (a gatePos, 0 = nothing released) the answer
-- was made under; 2147483647 marks an internal job's unlimited horizon.
-- `gate_pos` is the exact horizon the worker ran with. `rung` is what made
-- the answer and `source` whether a model wrote it or it is passages only.
CREATE TABLE trevor_answers (
    id         UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    version    TEXT        NOT NULL,
    server     TEXT        NOT NULL,
    qkey       TEXT        NOT NULL,
    era_pos    INTEGER     NOT NULL,
    gate_pos   INTEGER,
    rung       TEXT        NOT NULL CHECK (rung IN ('cf', 'cpu', 'passages')),
    source     TEXT        NOT NULL CHECK (source IN ('live', 'passages')),
    question   TEXT        NOT NULL,
    answer     JSONB       NOT NULL,
    hits       BIGINT      NOT NULL DEFAULT 0,
    -- False for an answer stored only for its job and never served from
    -- the cache: a passages-only answer, or one from a corpus version other
    -- than the job's.
    cached     BOOLEAN     NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- The cache key, over served answers only.
CREATE UNIQUE INDEX trevor_answers_key ON trevor_answers (version, server, qkey, era_pos) WHERE cached;
-- Precompute picks the most-asked questions of the active version.
CREATE INDEX trevor_answers_hits ON trevor_answers (server, version, hits DESC);

-- `rung` and `profile` are what the backend picked for the job (contract
-- section 2); `horizon_pos` is the gatePos horizon the worker receives, NULL
-- only for an internal job (no limit).
CREATE TABLE trevor_jobs (
    id          UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    kind        TEXT        NOT NULL DEFAULT 'ask' CHECK (kind IN ('ask', 'update')),
    user_id     UUID        REFERENCES users (id) ON DELETE SET NULL,
    question    TEXT        NOT NULL,
    qkey        TEXT        NOT NULL,
    horizon_pos INTEGER,
    server      TEXT        NOT NULL,
    -- The active corpus version when the job was queued.
    corpus_version TEXT,
    rung        TEXT        NOT NULL DEFAULT 'passages' CHECK (rung IN ('cf', 'cpu', 'passages')),
    profile     TEXT        NOT NULL DEFAULT 'full' CHECK (profile IN ('full', 'slim')),
    status      TEXT        NOT NULL DEFAULT 'queued'
                            CHECK (status IN ('queued', 'running', 'done', 'failed')),
    worker_id   TEXT,
    error       TEXT,
    attempts    SMALLINT    NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- When the job last entered the queue (a requeue resets it): the TTL clock.
    queued_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    answer_id   UUID        REFERENCES trevor_answers (id) ON DELETE SET NULL
);
-- The queue: the oldest queued job of a server, claimed with SKIP LOCKED.
CREATE INDEX trevor_jobs_queue ON trevor_jobs (server, status, created_at);
-- The per-user cap counts a user's open jobs.
CREATE INDEX trevor_jobs_user_open ON trevor_jobs (user_id) WHERE status IN ('queued', 'running');
-- A second ask of a question already in the queue joins that job; unique,
-- so two concurrent identical asks cannot both enqueue.
CREATE UNIQUE INDEX trevor_jobs_open_key ON trevor_jobs (server, qkey, horizon_pos) WHERE status IN ('queued', 'running');
-- The CPU rung's depth and the per-rung ETA read jobs by rung.
CREATE INDEX trevor_jobs_rung ON trevor_jobs (rung, status, finished_at);

-- Each worker's last heartbeat (contract section 3). Online = `last_seen`
-- within 75 s; the row outlives that, so the rungs a worker last reported
-- still steer the rung choice while it restarts.
CREATE TABLE trevor_workers (
    worker_id      TEXT        NOT NULL PRIMARY KEY,
    last_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    rungs          JSONB       NOT NULL DEFAULT '{}'::jsonb,
    corpus_version TEXT,
    busy_job_id    UUID
);

-- Cloudflare Workers AI Neurons spent per UTC day by `cf` answers.
CREATE TABLE trevor_neurons (
    day     DATE    NOT NULL PRIMARY KEY,
    neurons INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE trevor_feedback (
    answer_id  UUID        NOT NULL REFERENCES trevor_answers (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vote       SMALLINT    NOT NULL CHECK (vote IN (-1, 0, 1)),
    note       TEXT        CHECK (note IS NULL OR length(note) <= 2000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (answer_id, user_id)
);
