ALTER TABLE user_status ADD COLUMN IF NOT EXISTS originite integer;

CREATE OR REPLACE VIEW v_user_profile AS
SELECT
    u.id, u.uid, u.nickname, u.level, u.avatar_id, u.secretary,
    u.secretary_skin_id, u.resume_id, u.role,
    s.code AS server,
    sc.total_score, sc.grade,
    us.public_profile, us.store_gacha, us.share_stats,
    st.exp, st.orundum, st.lmd, st.sanity, st.max_sanity,
    st.gacha_tickets, st.ten_pull_tickets, st.monthly_sub_end,
    st.register_ts, st.last_online_ts, st.resume, st.friend_num_limit,
    (SELECT COUNT(*) FROM user_operators uo WHERE uo.user_id = u.id) AS operator_count,
    (SELECT COUNT(*) FROM user_items ui WHERE ui.user_id = u.id) AS item_count,
    (SELECT COUNT(*) FROM user_skins sk WHERE sk.user_id = u.id) AS skin_count,
    u.nick_number,
    (SELECT COUNT(*) FROM user_skins sk
       WHERE sk.user_id = u.id AND sk.skin_id LIKE '%@%') AS non_default_skin_count,
    ck.cumulative_signin,
    u.updated_at,
    st.originite
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
LEFT JOIN user_settings us ON us.user_id = u.id
LEFT JOIN user_status st ON st.user_id = u.id
LEFT JOIN user_checkin ck ON ck.user_id = u.id;
