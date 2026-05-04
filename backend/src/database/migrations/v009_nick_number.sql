-- ═══════════════════════════════════════════════════════════════
-- NICK NUMBER
--
-- Arknights identifies each Doctor by `nickName` plus a numeric
-- discriminator (`nickNumber`) — e.g. `Amiya#1234`. The syncData
-- payload exposes it as `status.nickNumber` (string of digits, may
-- have leading zeros so we store as VARCHAR).
-- ═══════════════════════════════════════════════════════════════

ALTER TABLE users ADD COLUMN IF NOT EXISTS nick_number VARCHAR(10);

-- Append nick_number to the user-profile view. CREATE OR REPLACE
-- VIEW only allows additions at the end of the column list.
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
    u.nick_number
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
LEFT JOIN user_settings us ON us.user_id = u.id
LEFT JOIN user_status st ON st.user_id = u.id;

-- Append nick_number to the leaderboard view.
CREATE OR REPLACE VIEW v_leaderboard AS
SELECT
    u.id, u.uid, u.nickname, u.level, u.avatar_id, u.secretary, u.secretary_skin_id,
    s.code AS server,
    sc.total_score, sc.grade,
    sc.operator_score, sc.stage_score, sc.roguelike_score,
    sc.sandbox_score, sc.medal_score, sc.base_score, sc.skin_score,
    RANK() OVER (ORDER BY sc.total_score DESC) AS rank_global,
    RANK() OVER (PARTITION BY u.server_id ORDER BY sc.total_score DESC) AS rank_server,
    u.nick_number
FROM users u
JOIN servers s ON u.server_id = s.id
LEFT JOIN user_scores sc ON u.id = sc.user_id
WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile = true);

-- Extend the sync procedure to accept p_nick_number and persist it
-- alongside the rest of the user record. Adding a parameter changes
-- the procedure signature, so drop the old 21-arg version first to
-- avoid an unwanted overload.
DROP PROCEDURE IF EXISTS sp_sync_user_data(
    VARCHAR, SMALLINT, VARCHAR, SMALLINT, VARCHAR, VARCHAR, VARCHAR, VARCHAR,
    JSONB, JSONB, JSONB, JSONB, JSONB, JSONB, JSONB, JSONB, JSONB, JSONB, JSONB,
    SMALLINT[], JSONB
);

CREATE OR REPLACE PROCEDURE sp_sync_user_data(
    p_uid VARCHAR,
    p_server_id SMALLINT,
    p_nickname VARCHAR,
    p_level SMALLINT,
    p_avatar_id VARCHAR,
    p_secretary VARCHAR,
    p_secretary_skin_id VARCHAR,
    p_resume_id VARCHAR,
    p_operators JSONB,
    p_skills JSONB,
    p_modules JSONB,
    p_items JSONB,
    p_skins JSONB,
    p_status JSONB,
    p_stages JSONB,
    p_roguelike JSONB,
    p_sandbox JSONB,
    p_medals JSONB,
    p_building JSONB,
    p_checkin SMALLINT[],
    p_supports JSONB,
    p_nick_number VARCHAR
)
LANGUAGE plpgsql AS $$
DECLARE
    v_user_id UUID;
BEGIN
    INSERT INTO users (uid, server_id, nickname, nick_number, level, avatar_id, secretary, secretary_skin_id, resume_id)
    VALUES (p_uid, p_server_id, p_nickname, p_nick_number, p_level, p_avatar_id, p_secretary, p_secretary_skin_id, p_resume_id)
    ON CONFLICT (uid, server_id) DO UPDATE SET
        nickname = EXCLUDED.nickname, nick_number = EXCLUDED.nick_number,
        level = EXCLUDED.level,
        avatar_id = EXCLUDED.avatar_id, secretary = EXCLUDED.secretary,
        secretary_skin_id = EXCLUDED.secretary_skin_id, resume_id = EXCLUDED.resume_id
    RETURNING id INTO v_user_id;

    INSERT INTO user_settings (user_id) VALUES (v_user_id) ON CONFLICT DO NOTHING;

    INSERT INTO user_status (user_id, exp, orundum, orundum_shard, lmd, sanity, max_sanity,
        gacha_tickets, ten_pull_tickets, classic_gacha_tickets, classic_ten_pull_tickets,
        recruit_permits, social_point, hgg_shard, lgg_shard, practice_tickets, gold,
        monthly_sub_end, register_ts, last_online_ts, main_stage_progress, resume, friend_num_limit)
    VALUES (v_user_id,
        (p_status->>'exp')::INT, (p_status->>'orundum')::INT, (p_status->>'orundum_shard')::INT,
        (p_status->>'lmd')::INT, (p_status->>'sanity')::SMALLINT, (p_status->>'max_sanity')::SMALLINT,
        (p_status->>'gacha_tickets')::INT, (p_status->>'ten_pull_tickets')::INT,
        (p_status->>'classic_gacha_tickets')::INT, (p_status->>'classic_ten_pull_tickets')::INT,
        (p_status->>'recruit_permits')::INT, (p_status->>'social_point')::INT,
        (p_status->>'hgg_shard')::INT, (p_status->>'lgg_shard')::INT,
        (p_status->>'practice_tickets')::INT, (p_status->>'gold')::INT,
        (p_status->>'monthly_sub_end')::BIGINT, (p_status->>'register_ts')::BIGINT,
        (p_status->>'last_online_ts')::BIGINT, p_status->>'main_stage_progress',
        p_status->>'resume', (p_status->>'friend_num_limit')::SMALLINT)
    ON CONFLICT (user_id) DO UPDATE SET
        exp = EXCLUDED.exp, orundum = EXCLUDED.orundum, orundum_shard = EXCLUDED.orundum_shard,
        lmd = EXCLUDED.lmd, sanity = EXCLUDED.sanity, max_sanity = EXCLUDED.max_sanity,
        gacha_tickets = EXCLUDED.gacha_tickets, ten_pull_tickets = EXCLUDED.ten_pull_tickets,
        classic_gacha_tickets = EXCLUDED.classic_gacha_tickets,
        classic_ten_pull_tickets = EXCLUDED.classic_ten_pull_tickets,
        recruit_permits = EXCLUDED.recruit_permits, social_point = EXCLUDED.social_point,
        hgg_shard = EXCLUDED.hgg_shard, lgg_shard = EXCLUDED.lgg_shard,
        practice_tickets = EXCLUDED.practice_tickets, gold = EXCLUDED.gold,
        monthly_sub_end = EXCLUDED.monthly_sub_end, register_ts = EXCLUDED.register_ts,
        last_online_ts = EXCLUDED.last_online_ts, main_stage_progress = EXCLUDED.main_stage_progress,
        resume = EXCLUDED.resume, friend_num_limit = EXCLUDED.friend_num_limit;

    DELETE FROM user_operators WHERE user_id = v_user_id;
    INSERT INTO user_operators (user_id, operator_id, elite, level, exp, potential, skill_level,
        favor_point, skin_id, default_skill, voice_lan, current_equip, current_tmpl, obtained_at)
    SELECT v_user_id, op->>'operator_id', (op->>'elite')::SMALLINT, (op->>'level')::SMALLINT,
           COALESCE((op->>'exp')::INT, 0), (op->>'potential')::SMALLINT, (op->>'skill_level')::SMALLINT,
           COALESCE((op->>'favor_point')::INT, 0), op->>'skin_id', COALESCE((op->>'default_skill')::SMALLINT, 0),
           op->>'voice_lan', op->>'current_equip', op->>'current_tmpl', (op->>'obtained_at')::BIGINT
    FROM jsonb_array_elements(p_operators) AS op;

    INSERT INTO user_operator_skills (user_id, operator_id, skill_index, specialize_level)
    SELECT v_user_id, sk->>'operator_id', (sk->>'skill_index')::SMALLINT, (sk->>'specialize_level')::SMALLINT
    FROM jsonb_array_elements(p_skills) AS sk;

    INSERT INTO user_operator_modules (user_id, operator_id, module_id, module_level, locked)
    SELECT v_user_id, m->>'operator_id', m->>'module_id', (m->>'module_level')::SMALLINT,
           COALESCE((m->>'locked')::BOOLEAN, false)
    FROM jsonb_array_elements(p_modules) AS m;

    DELETE FROM user_items WHERE user_id = v_user_id;
    INSERT INTO user_items (user_id, item_id, quantity)
    SELECT v_user_id, i->>'item_id', (i->>'quantity')::INT
    FROM jsonb_array_elements(p_items) AS i
    WHERE (i->>'quantity')::INT > 0;

    DELETE FROM user_skins WHERE user_id = v_user_id;
    INSERT INTO user_skins (user_id, skin_id, obtained_at)
    SELECT v_user_id, sk->>'skin_id', (sk->>'obtained_at')::BIGINT
    FROM jsonb_array_elements(p_skins) AS sk;

    INSERT INTO user_stage_progress (user_id, stages) VALUES (v_user_id, p_stages)
    ON CONFLICT (user_id) DO UPDATE SET stages = EXCLUDED.stages;

    DELETE FROM user_roguelike_progress WHERE user_id = v_user_id;
    INSERT INTO user_roguelike_progress (user_id, theme_id, progress)
    SELECT v_user_id, r->>'theme_id', r->'progress'
    FROM jsonb_array_elements(p_roguelike) AS r;

    INSERT INTO user_sandbox_progress (user_id, progress) VALUES (v_user_id, p_sandbox)
    ON CONFLICT (user_id) DO UPDATE SET progress = EXCLUDED.progress;

    DELETE FROM user_medals WHERE user_id = v_user_id;
    INSERT INTO user_medals (user_id, medal_id, val, first_ts, reach_ts)
    SELECT v_user_id, m->>'medal_id', m->'val', (m->>'first_ts')::BIGINT, (m->>'reach_ts')::BIGINT
    FROM jsonb_array_elements(p_medals) AS m;

    INSERT INTO user_building (user_id, data) VALUES (v_user_id, p_building)
    ON CONFLICT (user_id) DO UPDATE SET data = EXCLUDED.data;

    INSERT INTO user_checkin (user_id, history) VALUES (v_user_id, p_checkin)
    ON CONFLICT (user_id) DO UPDATE SET history = EXCLUDED.history;

    DELETE FROM user_support_units WHERE user_id = v_user_id;
    INSERT INTO user_support_units (user_id, slot, operator_id, skin_id, skill_index, current_equip)
    SELECT v_user_id,
           (s->>'slot')::SMALLINT,
           s->>'operator_id',
           s->>'skin_id',
           COALESCE((s->>'skill_index')::SMALLINT, 0),
           s->>'current_equip'
    FROM jsonb_array_elements(p_supports) AS s
    WHERE s->>'operator_id' IS NOT NULL;
END;
$$;
