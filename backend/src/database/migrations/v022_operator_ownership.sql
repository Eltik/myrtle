-- ═══════════════════════════════════════════════════════════════
-- OPERATOR OWNERSHIP STATS (precomputed aggregate)
-- ═══════════════════════════════════════════════════════════════
-- Per-server, per-operator ownership tallies, refreshed by a periodic
-- background job so reads stay an indexed range scan over a small table rather
-- than an aggregate over the full roster on every request.
--
-- Keyed per server because rosters and the eligible population differ by
-- region. Counts are stored raw (not a precomputed percentage) so callers can
-- derive the ratio and the storage format never depends on display choices.
CREATE TABLE operator_ownership_stats (
    server_id    SMALLINT NOT NULL REFERENCES servers(id),
    operator_id  VARCHAR(50) NOT NULL,       -- e.g. "char_002_amiya"
    owners       INT NOT NULL DEFAULT 0,     -- eligible users on this server who own it
    population   INT NOT NULL DEFAULT 0,     -- eligible users on this server (the denominator)
    computed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (server_id, operator_id)
);

-- Reverse lookup: "which servers / how many own this one operator".
CREATE INDEX idx_oos_operator ON operator_ownership_stats(operator_id);
