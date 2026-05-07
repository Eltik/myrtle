CREATE TABLE leaderboard_snapshots (
    id          BIGSERIAL PRIMARY KEY,
    taken_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_lb_snapshots_taken_at ON leaderboard_snapshots(taken_at DESC);

CREATE TABLE leaderboard_snapshot_entries (
    snapshot_id  BIGINT NOT NULL REFERENCES leaderboard_snapshots(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id    SMALLINT NOT NULL REFERENCES servers(id),
    rank_global  INT NOT NULL,
    rank_server  INT NOT NULL,
    total_score  DOUBLE PRECISION,
    PRIMARY KEY (snapshot_id, user_id)
);

CREATE INDEX idx_lb_entries_user_snapshot
    ON leaderboard_snapshot_entries(user_id, snapshot_id DESC);

CREATE INDEX idx_lb_entries_snapshot_server
    ON leaderboard_snapshot_entries(snapshot_id, server_id);
