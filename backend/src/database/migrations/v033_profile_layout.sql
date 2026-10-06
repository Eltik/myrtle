--
-- A profile owner chooses the order of their profile's tabs and which of them
-- visitors may open.
--
-- `user_settings.profile_layout` is a jsonb OBJECT, so later phases can add
-- sibling keys (`showcase`, `background`) beside `tabs` without a migration.
-- This phase defines only `tabs`, an ordered list of `{ "id", "visible" }`.
-- The backend normalizes the list on every write and every read
-- (`models::profile_layout`), so an id dropped from the code disappears and a
-- tab added later shows up visible, with no rewrite of stored rows.
--
-- NULL means the owner never customized: today's order, every tab visible,
-- and the client keeps its old default-tab behavior. That is the kill switch,
-- so the column has no default and no existing row changes.
--
ALTER TABLE user_settings ADD COLUMN profile_layout jsonb;

-- CREATE OR REPLACE VIEW only allows appending columns, hence last position.
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
    st.originite,
    us.profile_layout
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
LEFT JOIN user_settings us ON us.user_id = u.id
LEFT JOIN user_status st ON st.user_id = u.id
LEFT JOIN user_checkin ck ON ck.user_id = u.id;
