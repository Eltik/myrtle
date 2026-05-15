-- ═══════════════════════════════════════════════════════════════
-- SPLIT OWNED-SKIN COUNT INTO TOTAL VS. NON-DEFAULT
--
-- The existing `skin_count` on v_user_profile is a raw COUNT(*)
-- over user_skins and therefore includes every default operator
-- skin. The profile UI and stats tab both want to display the
-- "outfits collected" number - i.e. non-default skins only, which
-- in Arknights are the skin_ids that contain an `@`. Adding a
-- separate column keeps the legacy total available while letting
-- the frontend report the user-facing number consistently.
--
-- CREATE OR REPLACE VIEW only allows appending columns, so the new
-- field goes at the end.
-- ═══════════════════════════════════════════════════════════════

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
       WHERE sk.user_id = u.id AND sk.skin_id LIKE '%@%') AS non_default_skin_count
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
LEFT JOIN user_settings us ON us.user_id = u.id
LEFT JOIN user_status st ON st.user_id = u.id;
