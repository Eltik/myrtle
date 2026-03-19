-- Enable trigram extension for fuzzy search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Users
CREATE INDEX idx_users_server ON users(server_id);
CREATE INDEX idx_users_nickname ON users USING gin(nickname gin_trgm_ops);

-- Scores (leaderboard)
CREATE INDEX idx_scores_total ON user_scores(total_score DESC);
CREATE INDEX idx_scores_composite ON user_scores(composite_score DESC);

-- Operators
CREATE INDEX idx_user_ops_operator ON user_operators(operator_id);

-- Gacha
CREATE INDEX idx_gacha_user_time ON gacha_records(user_id, pull_timestamp DESC);
CREATE INDEX idx_gacha_user_rarity ON gacha_records(user_id, rarity);

-- Tier lists
CREATE INDEX idx_tier_lists_type ON tier_lists(list_type) WHERE is_active = true;
CREATE INDEX idx_tier_lists_slug ON tier_lists(slug);

-- Medals (for medal score queries)
CREATE INDEX idx_user_medals_user ON user_medals(user_id);

-- Skins (for skin count/score)
CREATE INDEX idx_user_skins_user ON user_skins(user_id);

-- Audit
CREATE INDEX idx_audit_table_time ON audit_log(table_name, changed_at DESC);
