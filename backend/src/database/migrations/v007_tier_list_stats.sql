CREATE TABLE tier_list_flairs (
    id              SMALLSERIAL PRIMARY KEY,
    code            VARCHAR(30) NOT NULL UNIQUE,
    label           VARCHAR(50) NOT NULL,
    color           VARCHAR(7),
    display_order   SMALLINT NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO tier_list_flairs (code, label, color, display_order) VALUES
    ('endgame',   'Endgame',   '#b91c1c', 10),
    ('beginner',  'Beginner',  '#15803d', 20),
    ('roguelike', 'Roguelike', '#7c3aed', 30),
    ('niche',     'Niche',     '#0369a1', 40),
    ('meta',      'Meta',      '#ca8a04', 50),
    ('event',     'Event',     '#be185d', 60),
    ('fun',       'Fun',       '#ea580c', 70);

ALTER TABLE tier_lists
    ADD COLUMN IF NOT EXISTS flair_id SMALLINT REFERENCES tier_list_flairs(id) ON DELETE SET NULL;

CREATE TABLE tier_list_stats (
    tier_list_id        UUID PRIMARY KEY REFERENCES tier_lists(id) ON DELETE CASCADE,
    view_count          BIGINT NOT NULL DEFAULT 0,
    unique_view_count   BIGINT NOT NULL DEFAULT 0,
    favorite_count      INT    NOT NULL DEFAULT 0,
    share_count         INT    NOT NULL DEFAULT 0,
    is_trending         BOOLEAN NOT NULL DEFAULT false,
    trending_score      DOUBLE PRECISION NOT NULL DEFAULT 0,
    views_last_24h      INT NOT NULL DEFAULT 0,
    views_last_7d       INT NOT NULL DEFAULT 0,
    last_viewed_at      TIMESTAMPTZ,
    stats_updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tier_list_view_events (
    id              BIGSERIAL PRIMARY KEY,
    tier_list_id    UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,  -- nullable: anon
    session_hash    CHAR(64),    -- SHA-256 of IP+UA salt, for anon dedupe
    viewed_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tier_list_favorites (
    tier_list_id UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL REFERENCES users(id)      ON DELETE CASCADE,
    favorited_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tier_list_id, user_id)
);

CREATE INDEX idx_tier_lists_flair           ON tier_lists(flair_id) WHERE is_active = true;
CREATE INDEX idx_tl_stats_trending          ON tier_list_stats(is_trending, trending_score DESC)
                                            WHERE is_trending = true;
CREATE INDEX idx_tl_stats_views             ON tier_list_stats(view_count DESC);
CREATE INDEX idx_tl_view_events_list_time   ON tier_list_view_events(tier_list_id, viewed_at DESC);
CREATE INDEX idx_tl_view_events_dedupe_user ON tier_list_view_events(tier_list_id, user_id, viewed_at DESC)
                                            WHERE user_id IS NOT NULL;
CREATE INDEX idx_tl_view_events_dedupe_sess ON tier_list_view_events(tier_list_id, session_hash, viewed_at DESC)
                                            WHERE session_hash IS NOT NULL;
CREATE INDEX idx_tl_favorites_user          ON tier_list_favorites(user_id);

INSERT INTO tier_list_stats (tier_list_id)
SELECT id FROM tier_lists
ON CONFLICT DO NOTHING;
