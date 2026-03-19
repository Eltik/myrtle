-- Leaderboard view
CREATE VIEW v_leaderboard AS
SELECT
    u.id, u.uid, u.nickname, u.level, u.avatar_id, u.secretary, u.secretary_skin_id,
    s.code AS server,
    sc.total_score, sc.composite_score, sc.grade,
    sc.operator_score, sc.stage_score, sc.roguelike_score,
    sc.sandbox_score, sc.medal_score, sc.base_score, sc.skin_score,
    RANK() OVER (ORDER BY sc.total_score DESC) AS rank_global,
    RANK() OVER (PARTITION BY u.server_id ORDER BY sc.total_score DESC) AS rank_server
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile = true);

-- User profile view
CREATE VIEW v_user_profile AS
SELECT
    u.id, u.uid, u.nickname, u.level, u.avatar_id, u.secretary,
    u.secretary_skin_id, u.resume_id, u.role,
    s.code AS server,
    sc.total_score, sc.grade, sc.composite_score, sc.breakdown,
    us.public_profile, us.store_gacha, us.share_stats,
    st.exp, st.orundum, st.lmd, st.sanity, st.max_sanity,
    st.gacha_tickets, st.ten_pull_tickets, st.monthly_sub_end,
    st.register_ts, st.last_online_ts, st.resume, st.friend_num_limit,
    (SELECT COUNT(*) FROM user_operators uo WHERE uo.user_id = u.id) AS operator_count,
    (SELECT COUNT(*) FROM user_items ui WHERE ui.user_id = u.id) AS item_count,
    (SELECT COUNT(*) FROM user_skins sk WHERE sk.user_id = u.id) AS skin_count
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
LEFT JOIN user_settings us ON us.user_id = u.id
LEFT JOIN user_status st ON st.user_id = u.id;

-- Operator roster view
CREATE VIEW v_user_roster AS
SELECT
    uo.user_id, uo.operator_id, uo.elite, uo.level, uo.exp,
    uo.potential, uo.skill_level, uo.favor_point, uo.skin_id,
    uo.default_skill, uo.voice_lan, uo.current_equip, uo.current_tmpl,
    uo.obtained_at,
    COALESCE(
        (SELECT jsonb_agg(jsonb_build_object("index", s.skill_index, "mastery", s.specialize_level)
         ORDER BY s.skill_index)
         FROM user_operator_skills s
         WHERE s.user_id = uo.user_id AND s.operator_id = uo.operator_id),
        "[]"::jsonb
    ) AS masteries,
    COALESCE(
        (SELECT jsonb_agg(jsonb_build_object("id", m.module_id, "level", m.module_level, "locked", m.locked))
         FROM user_operator_modules m
         WHERE m.user_id = uo.user_id AND m.operator_id = uo.operator_id),
        "[]"::jsonb
    ) AS modules
FROM user_operators uo;

-- Gacha stats view
CREATE VIEW v_gacha_stats AS
SELECT
    user_id,
    COUNT(*) AS total_pulls,
    COUNT(*) FILTER (WHERE rarity = 6) AS six_star_count,
    COUNT(*) FILTER (WHERE rarity = 5) AS five_star_count,
    COUNT(*) FILTER (WHERE rarity = 4) AS four_star_count,
    MIN(pull_timestamp) AS first_pull,
    MAX(pull_timestamp) AS last_pull
FROM gacha_records
GROUP BY user_id;
