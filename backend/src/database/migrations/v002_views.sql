-- ============================================================================
-- BASELINE SCHEMA — VIEWS
--
-- Generated from `pg_dump --schema-only` of the migrated v001..v022 schema,
-- split by object type. Runs only on a fresh database (existing deployments
-- already have these objects recorded under the original migration names).
-- Generated artifact — do not hand-edit; add new changes as later migrations.
-- ============================================================================

--
-- Name: v_gacha_stats; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_gacha_stats AS
 SELECT gacha_records.user_id,
    count(*) AS total_pulls,
    count(*) FILTER (WHERE (gacha_records.rarity = 6)) AS six_star_count,
    count(*) FILTER (WHERE (gacha_records.rarity = 5)) AS five_star_count,
    count(*) FILTER (WHERE (gacha_records.rarity = 4)) AS four_star_count,
    min(gacha_records.pull_timestamp) AS first_pull,
    max(gacha_records.pull_timestamp) AS last_pull
   FROM public.gacha_records
  GROUP BY gacha_records.user_id;


--
-- Name: v_leaderboard; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_leaderboard AS
 SELECT u.id,
    u.uid,
    u.nickname,
    u.level,
    u.avatar_id,
    u.secretary,
    u.secretary_skin_id,
    s.code AS server,
    sc.total_score,
    sc.grade,
    sc.operator_score,
    sc.stage_score,
    sc.roguelike_score,
    sc.sandbox_score,
    sc.medal_score,
    sc.base_score,
    sc.skin_score,
    rank() OVER (ORDER BY sc.total_score DESC) AS rank_global,
    rank() OVER (PARTITION BY u.server_id ORDER BY sc.total_score DESC) AS rank_server,
    u.nick_number
   FROM ((public.users u
     JOIN public.servers s ON ((u.server_id = s.id)))
     LEFT JOIN public.user_scores sc ON ((u.id = sc.user_id)))
  WHERE (EXISTS ( SELECT 1
           FROM public.user_settings us
          WHERE ((us.user_id = u.id) AND (us.public_profile = true))));


--
-- Name: v_user_profile; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_user_profile AS
 SELECT u.id,
    u.uid,
    u.nickname,
    u.level,
    u.avatar_id,
    u.secretary,
    u.secretary_skin_id,
    u.resume_id,
    u.role,
    s.code AS server,
    sc.total_score,
    sc.grade,
    us.public_profile,
    us.store_gacha,
    us.share_stats,
    st.exp,
    st.orundum,
    st.lmd,
    st.sanity,
    st.max_sanity,
    st.gacha_tickets,
    st.ten_pull_tickets,
    st.monthly_sub_end,
    st.register_ts,
    st.last_online_ts,
    st.resume,
    st.friend_num_limit,
    ( SELECT count(*) AS count
           FROM public.user_operators uo
          WHERE (uo.user_id = u.id)) AS operator_count,
    ( SELECT count(*) AS count
           FROM public.user_items ui
          WHERE (ui.user_id = u.id)) AS item_count,
    ( SELECT count(*) AS count
           FROM public.user_skins sk
          WHERE (sk.user_id = u.id)) AS skin_count,
    u.nick_number,
    ( SELECT count(*) AS count
           FROM public.user_skins sk
          WHERE ((sk.user_id = u.id) AND ((sk.skin_id)::text ~~ '%@%'::text))) AS non_default_skin_count
   FROM ((((public.users u
     JOIN public.servers s ON ((u.server_id = s.id)))
     LEFT JOIN public.user_scores sc ON ((u.id = sc.user_id)))
     LEFT JOIN public.user_settings us ON ((us.user_id = u.id)))
     LEFT JOIN public.user_status st ON ((st.user_id = u.id)));


--
-- Name: v_user_roster; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_user_roster AS
 SELECT uo.user_id,
    uo.operator_id,
    uo.elite,
    uo.level,
    uo.exp,
    uo.potential,
    uo.skill_level,
    uo.favor_point,
    uo.skin_id,
    uo.default_skill,
    uo.voice_lan,
    uo.current_equip,
    uo.current_tmpl,
    uo.obtained_at,
    COALESCE(( SELECT jsonb_agg(jsonb_build_object('index', s.skill_index, 'mastery', s.specialize_level) ORDER BY s.skill_index) AS jsonb_agg
           FROM public.user_operator_skills s
          WHERE ((s.user_id = uo.user_id) AND ((s.operator_id)::text = (uo.operator_id)::text))), '[]'::jsonb) AS masteries,
    COALESCE(( SELECT jsonb_agg(jsonb_build_object('id', m.module_id, 'level', m.module_level, 'locked', m.locked)) AS jsonb_agg
           FROM public.user_operator_modules m
          WHERE ((m.user_id = uo.user_id) AND ((m.operator_id)::text = (uo.operator_id)::text))), '[]'::jsonb) AS modules
   FROM public.user_operators uo;
