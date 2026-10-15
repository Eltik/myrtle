--
-- The score leaderboard leaves out the level of a player who hid it from
-- their profile header (`profile_layout.header.hidden` holds "level"), as
-- `/get-user` nulls it for a visitor. The row stays ranked: the board ranks
-- score, which the level does not decide, so only the column goes NULL.
--
-- The expression is `ProfileHeaderField::Level.shown_sql("u.id", "u.level")`,
-- pinned by `the_leaderboard_view_masks_a_hidden_level`. A player with no
-- `header` key, or no layout, hides nothing, so every existing row reads as
-- before. Every other column, the gate and the ranks are v034's, unchanged.
--
CREATE OR REPLACE VIEW v_leaderboard AS
SELECT
    u.id, u.uid, u.nickname,
    CASE WHEN NOT EXISTS (SELECT 1 FROM user_settings hs WHERE hs.user_id = u.id AND hs.profile_layout -> 'header' -> 'hidden' @> '["level"]'::jsonb) THEN u.level END AS level,
    u.avatar_id, u.secretary, u.secretary_skin_id,
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
