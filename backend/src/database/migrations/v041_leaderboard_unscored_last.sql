--
-- An ungraded public player ranks after every graded one. Such a player
-- (signed up, never scored) has no `user_scores` row, so the LEFT JOIN gives
-- a NULL `total_score`, and Postgres sorts NULL FIRST under DESC: v038's
-- `rank()` put every one of them at rank 1, above the best graded player,
-- and the leaderboard snapshot job copied that rank 1 into history. With
-- `DESC NULLS LAST` in both windows, two graded players at 900 and 500 and
-- one ungraded player rank 1, 2, 3 (tests/leaderboard_rank_test.rs). All
-- ungraded players tie at the rank after the last graded one.
--
-- What did NOT change: every column, the public-profile and Score-tab gate,
-- and the hidden-level mask are v038's, byte for byte. `rank_global` is still
-- never NULL, so the `rank_global IS NOT NULL` filters in
-- database/queries/score.rs still keep ungraded players in the board, the
-- count and the snapshot, now at the bottom instead of the top.
--
-- Named v041, not v039: the unmerged `trevor` branch already ships
-- v039_trevor and v040_trevor_bank. `_migrations` is keyed by name, so either
-- merge order applies all three; v041 only needs to run after v038.
--
CREATE OR REPLACE VIEW v_leaderboard AS
SELECT
    u.id, u.uid, u.nickname,
    CASE WHEN NOT EXISTS (SELECT 1 FROM user_settings hs WHERE hs.user_id = u.id AND hs.profile_layout -> 'header' -> 'hidden' @> '["level"]'::jsonb) THEN u.level END AS level,
    u.avatar_id, u.secretary, u.secretary_skin_id,
    s.code AS server,
    sc.total_score, sc.grade, sc.operator_score, sc.stage_score, sc.roguelike_score,
    sc.sandbox_score, sc.medal_score, sc.base_score, sc.skin_score,
    rank() OVER (ORDER BY sc.total_score DESC NULLS LAST) AS rank_global,
    rank() OVER (PARTITION BY u.server_id ORDER BY sc.total_score DESC NULLS LAST) AS rank_server,
    u.nick_number
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
WHERE EXISTS (
    SELECT 1 FROM user_settings us
    WHERE us.user_id = u.id
      AND us.public_profile = true
      AND NOT COALESCE(us.profile_layout -> 'tabs' @> '[{"id":"score","visible":false}]'::jsonb, false)
);
