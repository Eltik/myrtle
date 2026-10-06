--
-- The score leaderboard drops a player who made their profile's Score tab
-- private, as `/get-user-score` refuses that tab's data to a visitor.
--
-- The gate sits in the view, not in each query, so every reader of it agrees:
-- the pages and their counts, the movers, the server distribution, a
-- player's standing and the snapshots `leaderboard/history` is built from.
-- `rank_global` and `rank_server` are computed after the gate, so the ranks
-- of the players still listed close over the hidden one instead of leaving a
-- gap that would give them away. That also means the owner has no standing
-- while the tab is private, exactly as with a private profile.
--
-- The predicate is `ProfileTabId::Score.visible_sql("us.profile_layout")`:
-- a NULL layout hides nothing, so every existing row is ranked as before.
--
CREATE OR REPLACE VIEW v_leaderboard AS
SELECT
    u.id, u.uid, u.nickname, u.level, u.avatar_id, u.secretary, u.secretary_skin_id,
    s.code AS server,
    sc.total_score, sc.grade, sc.operator_score, sc.stage_score, sc.roguelike_score,
    sc.sandbox_score, sc.medal_score, sc.base_score, sc.skin_score,
    rank() OVER (ORDER BY sc.total_score DESC) AS rank_global,
    rank() OVER (PARTITION BY u.server_id ORDER BY sc.total_score DESC) AS rank_server,
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
