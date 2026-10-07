-- Trevor, the story question answerer (design: trevor/design/trevor-integration.md).
--
-- The VPS holds only plumbing: the published corpus versions and their panel
-- JSON, a job queue a GPU worker pulls from, the answers it wrote back, and
-- reader feedback. Every row carries `server` ('en' now, 'cn' later), so a
-- second corpus is a second version stream, not a redesign.
--
-- `horizon` is the spoiler horizon (a story id): '' means "no horizon", so the
-- unique key on answers treats it as a value rather than as a NULL that never
-- collides.

CREATE TABLE trevor_versions (
    version      TEXT        NOT NULL,
    server       TEXT        NOT NULL,
    built_at     TIMESTAMPTZ,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    gate_report  JSONB,
    manifest     JSONB,
    active       BOOLEAN     NOT NULL DEFAULT false,
    PRIMARY KEY (version, server)
);
-- At most one active version per server; activation flips it in one transaction.
CREATE UNIQUE INDEX trevor_versions_one_active ON trevor_versions (server) WHERE active;

-- The deterministic per-story panels of one version (no model at read time).
CREATE TABLE trevor_panels (
    version  TEXT  NOT NULL,
    server   TEXT  NOT NULL,
    story_id TEXT  NOT NULL,
    panels   JSONB NOT NULL,
    PRIMARY KEY (version, server, story_id),
    FOREIGN KEY (version, server) REFERENCES trevor_versions (version, server) ON DELETE CASCADE
);

CREATE TABLE trevor_answers (
    id         UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    version    TEXT        NOT NULL,
    server     TEXT        NOT NULL,
    qkey       TEXT        NOT NULL,
    horizon    TEXT        NOT NULL DEFAULT '',
    question   TEXT        NOT NULL,
    answer     JSONB       NOT NULL,
    hits       BIGINT      NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT trevor_answers_key UNIQUE (version, server, qkey, horizon)
);
-- Precompute picks the most-asked questions of the active version.
CREATE INDEX trevor_answers_hits ON trevor_answers (server, version, hits DESC);

CREATE TABLE trevor_jobs (
    id          UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    kind        TEXT        NOT NULL DEFAULT 'ask' CHECK (kind IN ('ask', 'update')),
    user_id     UUID        REFERENCES users (id) ON DELETE SET NULL,
    question    TEXT        NOT NULL,
    qkey        TEXT        NOT NULL,
    horizon     TEXT        NOT NULL DEFAULT '',
    server      TEXT        NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'queued'
                            CHECK (status IN ('queued', 'running', 'done', 'failed')),
    worker_id   TEXT,
    error       TEXT,
    attempts    SMALLINT    NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    answer_id   UUID        REFERENCES trevor_answers (id) ON DELETE SET NULL
);
-- The queue: the oldest queued job of a server, claimed with SKIP LOCKED.
CREATE INDEX trevor_jobs_queue ON trevor_jobs (server, status, created_at);
-- The per-user cap counts a user's open jobs.
CREATE INDEX trevor_jobs_user_open ON trevor_jobs (user_id) WHERE status IN ('queued', 'running');
-- A second ask of a question already in the queue joins that job.
CREATE INDEX trevor_jobs_open_key ON trevor_jobs (server, qkey, horizon) WHERE status IN ('queued', 'running');

CREATE TABLE trevor_feedback (
    answer_id  UUID        NOT NULL REFERENCES trevor_answers (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vote       SMALLINT    NOT NULL CHECK (vote IN (-1, 0, 1)),
    note       TEXT        CHECK (note IS NULL OR length(note) <= 2000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (answer_id, user_id)
);
