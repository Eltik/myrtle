-- ============================================================================
-- BASELINE SCHEMA — TABLES, SEQUENCES, CONSTRAINTS & EXTENSIONS
--
-- Generated from `pg_dump --schema-only` of the migrated v001..v022 schema,
-- split by object type. Runs only on a fresh database (existing deployments
-- already have these objects recorded under the original migration names).
-- Generated artifact — do not hand-edit; add new changes as later migrations.
-- ============================================================================

--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;


--
-- Name: audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.audit_log (
    id bigint NOT NULL,
    table_name character varying(50) NOT NULL,
    record_id text NOT NULL,
    action character varying(10) NOT NULL,
    old_data jsonb,
    new_data jsonb,
    changed_by uuid,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: audit_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.audit_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: audit_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.audit_log_id_seq OWNED BY public.audit_log.id;


--
-- Name: gacha_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.gacha_records (
    id bigint NOT NULL,
    user_id uuid NOT NULL,
    char_id character varying(50) NOT NULL,
    pool_id character varying(50) NOT NULL,
    rarity smallint NOT NULL,
    pull_timestamp bigint NOT NULL,
    pool_name character varying(100),
    gacha_type character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    batch_index smallint DEFAULT 0 NOT NULL
);


--
-- Name: gacha_records_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.gacha_records_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: gacha_records_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.gacha_records_id_seq OWNED BY public.gacha_records.id;


--
-- Name: leaderboard_snapshot_entries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leaderboard_snapshot_entries (
    snapshot_id bigint NOT NULL,
    user_id uuid NOT NULL,
    server_id smallint NOT NULL,
    rank_global integer NOT NULL,
    rank_server integer NOT NULL,
    total_score double precision
);


--
-- Name: leaderboard_snapshots; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leaderboard_snapshots (
    id bigint NOT NULL,
    taken_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: leaderboard_snapshots_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.leaderboard_snapshots_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: leaderboard_snapshots_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.leaderboard_snapshots_id_seq OWNED BY public.leaderboard_snapshots.id;


--
-- Name: operator_notes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.operator_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    operator_id character varying(50) NOT NULL,
    pros text,
    cons text,
    notes text,
    trivia text,
    summary character varying(500),
    tags jsonb DEFAULT '[]'::jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: operator_notes_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.operator_notes_audit_log (
    id bigint NOT NULL,
    note_id uuid NOT NULL,
    field_name character varying(50) NOT NULL,
    old_value text,
    new_value text,
    changed_by uuid NOT NULL,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: operator_notes_audit_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.operator_notes_audit_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: operator_notes_audit_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.operator_notes_audit_log_id_seq OWNED BY public.operator_notes_audit_log.id;


--
-- Name: operator_ownership_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.operator_ownership_stats (
    server_id smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    owners integer DEFAULT 0 NOT NULL,
    population integer DEFAULT 0 NOT NULL,
    computed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: operator_plans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.operator_plans (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    operator_id character varying(50) NOT NULL,
    target_elite smallint DEFAULT 0 NOT NULL,
    target_level smallint DEFAULT 1 NOT NULL,
    target_skill_level smallint DEFAULT 1 NOT NULL,
    target_skills jsonb DEFAULT '[]'::jsonb NOT NULL,
    target_modules jsonb DEFAULT '[]'::jsonb NOT NULL,
    display_on_profile boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: plan_group_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plan_group_members (
    plan_group_id uuid NOT NULL,
    operator_plan_id uuid NOT NULL
);


--
-- Name: plan_groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plan_groups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: servers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.servers (
    id smallint NOT NULL,
    code character varying(4) NOT NULL,
    name character varying(20) NOT NULL
);


--
-- Name: tier_list_favorites; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_favorites (
    tier_list_id uuid NOT NULL,
    user_id uuid NOT NULL,
    favorited_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_flairs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_flairs (
    id smallint NOT NULL,
    code character varying(30) NOT NULL,
    label character varying(50) NOT NULL,
    color character varying(7),
    display_order smallint DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_flairs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tier_list_flairs_id_seq
    AS smallint
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tier_list_flairs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tier_list_flairs_id_seq OWNED BY public.tier_list_flairs.id;


--
-- Name: tier_list_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_permissions (
    tier_list_id uuid NOT NULL,
    user_id uuid NOT NULL,
    permission character varying(20) NOT NULL,
    granted_by uuid,
    granted_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_stats; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_stats (
    tier_list_id uuid NOT NULL,
    view_count bigint DEFAULT 0 NOT NULL,
    unique_view_count bigint DEFAULT 0 NOT NULL,
    favorite_count integer DEFAULT 0 NOT NULL,
    share_count integer DEFAULT 0 NOT NULL,
    is_trending boolean DEFAULT false NOT NULL,
    trending_score double precision DEFAULT 0 NOT NULL,
    views_last_24h integer DEFAULT 0 NOT NULL,
    views_last_7d integer DEFAULT 0 NOT NULL,
    last_viewed_at timestamp with time zone,
    stats_updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    tier_list_id uuid NOT NULL,
    version integer NOT NULL,
    snapshot jsonb NOT NULL,
    changelog text,
    published_by uuid,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_view_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_list_view_events (
    id bigint NOT NULL,
    tier_list_id uuid NOT NULL,
    user_id uuid,
    session_hash character(64),
    viewed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tier_list_view_events_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tier_list_view_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tier_list_view_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tier_list_view_events_id_seq OWNED BY public.tier_list_view_events.id;


--
-- Name: tier_lists; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_lists (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    slug character varying(100) NOT NULL,
    description text,
    list_type character varying(20) DEFAULT 'official'::character varying NOT NULL,
    created_by uuid,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    flair_id smallint,
    is_listed boolean DEFAULT true NOT NULL
);


--
-- Name: tier_placements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tier_placements (
    tier_id uuid NOT NULL,
    operator_id character varying(50) NOT NULL,
    sub_order smallint DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    description text
);


--
-- Name: tiers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tiers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    tier_list_id uuid NOT NULL,
    name character varying(40) NOT NULL,
    display_order smallint NOT NULL,
    color character varying(7),
    description text
);


--
-- Name: user_building; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_building (
    user_id uuid NOT NULL,
    data jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: user_checkin; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_checkin (
    user_id uuid NOT NULL,
    history smallint[] DEFAULT '{}'::smallint[] NOT NULL
);


--
-- Name: user_enemy_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_enemy_progress (
    user_id uuid NOT NULL,
    enemies jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: user_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_items (
    user_id uuid NOT NULL,
    item_id character varying(50) NOT NULL,
    quantity integer DEFAULT 0 NOT NULL
);


--
-- Name: user_medals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_medals (
    user_id uuid NOT NULL,
    medal_id character varying(50) NOT NULL,
    val jsonb,
    first_ts bigint,
    reach_ts bigint
);


--
-- Name: user_operator_modules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_operator_modules (
    user_id uuid NOT NULL,
    operator_id character varying(50) NOT NULL,
    module_id character varying(50) NOT NULL,
    module_level smallint DEFAULT 0 NOT NULL,
    locked boolean DEFAULT false NOT NULL
);


--
-- Name: user_operator_skills; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_operator_skills (
    user_id uuid NOT NULL,
    operator_id character varying(50) NOT NULL,
    skill_index smallint NOT NULL,
    specialize_level smallint DEFAULT 0 NOT NULL
);


--
-- Name: user_operators; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_operators (
    user_id uuid NOT NULL,
    operator_id character varying(50) NOT NULL,
    elite smallint DEFAULT 0 NOT NULL,
    level smallint DEFAULT 1 NOT NULL,
    exp integer DEFAULT 0 NOT NULL,
    potential smallint DEFAULT 0 NOT NULL,
    skill_level smallint DEFAULT 1 NOT NULL,
    favor_point integer DEFAULT 0 NOT NULL,
    skin_id character varying(50),
    default_skill smallint DEFAULT 0,
    voice_lan character varying(20),
    current_equip character varying(50),
    current_tmpl character varying(50),
    obtained_at bigint
);


--
-- Name: user_roguelike_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_roguelike_progress (
    user_id uuid NOT NULL,
    theme_id character varying(20) NOT NULL,
    progress jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: user_sandbox_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sandbox_progress (
    user_id uuid NOT NULL,
    progress jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: user_scores; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_scores (
    user_id uuid NOT NULL,
    total_score double precision DEFAULT 0 NOT NULL,
    operator_score double precision DEFAULT 0 NOT NULL,
    stage_score double precision DEFAULT 0 NOT NULL,
    roguelike_score double precision DEFAULT 0 NOT NULL,
    sandbox_score double precision DEFAULT 0 NOT NULL,
    medal_score double precision DEFAULT 0 NOT NULL,
    base_score double precision DEFAULT 0 NOT NULL,
    skin_score double precision DEFAULT 0 NOT NULL,
    grade character varying(5),
    calculated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_settings (
    user_id uuid NOT NULL,
    public_profile boolean DEFAULT true NOT NULL,
    store_gacha boolean DEFAULT true NOT NULL,
    share_stats boolean DEFAULT true NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_skins; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_skins (
    user_id uuid NOT NULL,
    skin_id character varying(50) NOT NULL,
    obtained_at bigint
);


--
-- Name: user_stage_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_stage_progress (
    user_id uuid NOT NULL,
    stages jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: user_status; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_status (
    user_id uuid NOT NULL,
    exp integer DEFAULT 0 NOT NULL,
    orundum integer DEFAULT 0 NOT NULL,
    orundum_shard integer DEFAULT 0 NOT NULL,
    lmd integer DEFAULT 0 NOT NULL,
    sanity smallint DEFAULT 0 NOT NULL,
    max_sanity smallint DEFAULT 0 NOT NULL,
    gacha_tickets integer DEFAULT 0 NOT NULL,
    ten_pull_tickets integer DEFAULT 0 NOT NULL,
    classic_gacha_tickets integer DEFAULT 0 NOT NULL,
    classic_ten_pull_tickets integer DEFAULT 0 NOT NULL,
    recruit_permits integer DEFAULT 0 NOT NULL,
    social_point integer DEFAULT 0 NOT NULL,
    hgg_shard integer DEFAULT 0 NOT NULL,
    lgg_shard integer DEFAULT 0 NOT NULL,
    practice_tickets integer DEFAULT 0 NOT NULL,
    gold integer DEFAULT 0 NOT NULL,
    monthly_sub_end bigint,
    register_ts bigint,
    last_online_ts bigint,
    main_stage_progress character varying(50),
    resume text,
    friend_num_limit smallint DEFAULT 0 NOT NULL
);


--
-- Name: user_support_units; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_support_units (
    user_id uuid NOT NULL,
    slot smallint NOT NULL,
    operator_id character varying(50) NOT NULL,
    skin_id character varying(50),
    skill_index smallint DEFAULT 0 NOT NULL,
    current_equip character varying(50)
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    uid character varying(20) NOT NULL,
    server_id smallint NOT NULL,
    nickname character varying(50),
    level smallint DEFAULT 0,
    role character varying(20) DEFAULT 'user'::character varying NOT NULL,
    avatar_id character varying(50),
    resume_id character varying(50),
    secretary character varying(50),
    secretary_skin_id character varying(50),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    nick_number character varying(10)
);


--
-- Name: audit_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_log ALTER COLUMN id SET DEFAULT nextval('public.audit_log_id_seq'::regclass);


--
-- Name: gacha_records id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gacha_records ALTER COLUMN id SET DEFAULT nextval('public.gacha_records_id_seq'::regclass);


--
-- Name: leaderboard_snapshots id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshots ALTER COLUMN id SET DEFAULT nextval('public.leaderboard_snapshots_id_seq'::regclass);


--
-- Name: operator_notes_audit_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes_audit_log ALTER COLUMN id SET DEFAULT nextval('public.operator_notes_audit_log_id_seq'::regclass);


--
-- Name: tier_list_flairs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_flairs ALTER COLUMN id SET DEFAULT nextval('public.tier_list_flairs_id_seq'::regclass);


--
-- Name: tier_list_view_events id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_view_events ALTER COLUMN id SET DEFAULT nextval('public.tier_list_view_events_id_seq'::regclass);


--
-- Name: audit_log audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_log
    ADD CONSTRAINT audit_log_pkey PRIMARY KEY (id);


--
-- Name: gacha_records gacha_records_batch_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gacha_records
    ADD CONSTRAINT gacha_records_batch_unique UNIQUE (user_id, pull_timestamp, char_id, pool_id, batch_index);


--
-- Name: gacha_records gacha_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gacha_records
    ADD CONSTRAINT gacha_records_pkey PRIMARY KEY (id);


--
-- Name: leaderboard_snapshot_entries leaderboard_snapshot_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshot_entries
    ADD CONSTRAINT leaderboard_snapshot_entries_pkey PRIMARY KEY (snapshot_id, user_id);


--
-- Name: leaderboard_snapshots leaderboard_snapshots_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshots
    ADD CONSTRAINT leaderboard_snapshots_pkey PRIMARY KEY (id);


--
-- Name: operator_notes_audit_log operator_notes_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes_audit_log
    ADD CONSTRAINT operator_notes_audit_log_pkey PRIMARY KEY (id);


--
-- Name: operator_notes operator_notes_operator_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes
    ADD CONSTRAINT operator_notes_operator_id_key UNIQUE (operator_id);


--
-- Name: operator_notes operator_notes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes
    ADD CONSTRAINT operator_notes_pkey PRIMARY KEY (id);


--
-- Name: operator_ownership_stats operator_ownership_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_ownership_stats
    ADD CONSTRAINT operator_ownership_stats_pkey PRIMARY KEY (server_id, operator_id);


--
-- Name: operator_plans operator_plans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_plans
    ADD CONSTRAINT operator_plans_pkey PRIMARY KEY (id);


--
-- Name: operator_plans operator_plans_user_id_operator_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_plans
    ADD CONSTRAINT operator_plans_user_id_operator_id_key UNIQUE (user_id, operator_id);


--
-- Name: plan_group_members plan_group_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_group_members
    ADD CONSTRAINT plan_group_members_pkey PRIMARY KEY (plan_group_id, operator_plan_id);


--
-- Name: plan_groups plan_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_groups
    ADD CONSTRAINT plan_groups_pkey PRIMARY KEY (id);


--
-- Name: plan_groups plan_groups_user_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_groups
    ADD CONSTRAINT plan_groups_user_id_name_key UNIQUE (user_id, name);


--
-- Name: servers servers_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.servers
    ADD CONSTRAINT servers_code_key UNIQUE (code);


--
-- Name: servers servers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.servers
    ADD CONSTRAINT servers_pkey PRIMARY KEY (id);


--
-- Name: tier_list_favorites tier_list_favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_favorites
    ADD CONSTRAINT tier_list_favorites_pkey PRIMARY KEY (tier_list_id, user_id);


--
-- Name: tier_list_flairs tier_list_flairs_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_flairs
    ADD CONSTRAINT tier_list_flairs_code_key UNIQUE (code);


--
-- Name: tier_list_flairs tier_list_flairs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_flairs
    ADD CONSTRAINT tier_list_flairs_pkey PRIMARY KEY (id);


--
-- Name: tier_list_permissions tier_list_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_permissions
    ADD CONSTRAINT tier_list_permissions_pkey PRIMARY KEY (tier_list_id, user_id, permission);


--
-- Name: tier_list_stats tier_list_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_stats
    ADD CONSTRAINT tier_list_stats_pkey PRIMARY KEY (tier_list_id);


--
-- Name: tier_list_versions tier_list_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_versions
    ADD CONSTRAINT tier_list_versions_pkey PRIMARY KEY (id);


--
-- Name: tier_list_versions tier_list_versions_tier_list_id_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_versions
    ADD CONSTRAINT tier_list_versions_tier_list_id_version_key UNIQUE (tier_list_id, version);


--
-- Name: tier_list_view_events tier_list_view_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_view_events
    ADD CONSTRAINT tier_list_view_events_pkey PRIMARY KEY (id);


--
-- Name: tier_lists tier_lists_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_lists
    ADD CONSTRAINT tier_lists_pkey PRIMARY KEY (id);


--
-- Name: tier_lists tier_lists_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_lists
    ADD CONSTRAINT tier_lists_slug_key UNIQUE (slug);


--
-- Name: tier_placements tier_placements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_placements
    ADD CONSTRAINT tier_placements_pkey PRIMARY KEY (tier_id, operator_id);


--
-- Name: tiers tiers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tiers
    ADD CONSTRAINT tiers_pkey PRIMARY KEY (id);


--
-- Name: tiers tiers_tier_list_id_display_order_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tiers
    ADD CONSTRAINT tiers_tier_list_id_display_order_key UNIQUE (tier_list_id, display_order);


--
-- Name: user_building user_building_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_building
    ADD CONSTRAINT user_building_pkey PRIMARY KEY (user_id);


--
-- Name: user_checkin user_checkin_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_checkin
    ADD CONSTRAINT user_checkin_pkey PRIMARY KEY (user_id);


--
-- Name: user_enemy_progress user_enemy_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_enemy_progress
    ADD CONSTRAINT user_enemy_progress_pkey PRIMARY KEY (user_id);


--
-- Name: user_items user_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_items
    ADD CONSTRAINT user_items_pkey PRIMARY KEY (user_id, item_id);


--
-- Name: user_medals user_medals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_medals
    ADD CONSTRAINT user_medals_pkey PRIMARY KEY (user_id, medal_id);


--
-- Name: user_operator_modules user_operator_modules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operator_modules
    ADD CONSTRAINT user_operator_modules_pkey PRIMARY KEY (user_id, operator_id, module_id);


--
-- Name: user_operator_skills user_operator_skills_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operator_skills
    ADD CONSTRAINT user_operator_skills_pkey PRIMARY KEY (user_id, operator_id, skill_index);


--
-- Name: user_operators user_operators_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operators
    ADD CONSTRAINT user_operators_pkey PRIMARY KEY (user_id, operator_id);


--
-- Name: user_roguelike_progress user_roguelike_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roguelike_progress
    ADD CONSTRAINT user_roguelike_progress_pkey PRIMARY KEY (user_id, theme_id);


--
-- Name: user_sandbox_progress user_sandbox_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sandbox_progress
    ADD CONSTRAINT user_sandbox_progress_pkey PRIMARY KEY (user_id);


--
-- Name: user_scores user_scores_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scores
    ADD CONSTRAINT user_scores_pkey PRIMARY KEY (user_id);


--
-- Name: user_settings user_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_pkey PRIMARY KEY (user_id);


--
-- Name: user_skins user_skins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_skins
    ADD CONSTRAINT user_skins_pkey PRIMARY KEY (user_id, skin_id);


--
-- Name: user_stage_progress user_stage_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_stage_progress
    ADD CONSTRAINT user_stage_progress_pkey PRIMARY KEY (user_id);


--
-- Name: user_status user_status_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_status
    ADD CONSTRAINT user_status_pkey PRIMARY KEY (user_id);


--
-- Name: user_support_units user_support_units_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_support_units
    ADD CONSTRAINT user_support_units_pkey PRIMARY KEY (user_id, slot);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_uid_server_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_uid_server_id_key UNIQUE (uid, server_id);


--
-- Name: gacha_records gacha_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gacha_records
    ADD CONSTRAINT gacha_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: leaderboard_snapshot_entries leaderboard_snapshot_entries_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshot_entries
    ADD CONSTRAINT leaderboard_snapshot_entries_server_id_fkey FOREIGN KEY (server_id) REFERENCES public.servers(id);


--
-- Name: leaderboard_snapshot_entries leaderboard_snapshot_entries_snapshot_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshot_entries
    ADD CONSTRAINT leaderboard_snapshot_entries_snapshot_id_fkey FOREIGN KEY (snapshot_id) REFERENCES public.leaderboard_snapshots(id) ON DELETE CASCADE;


--
-- Name: leaderboard_snapshot_entries leaderboard_snapshot_entries_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leaderboard_snapshot_entries
    ADD CONSTRAINT leaderboard_snapshot_entries_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: operator_notes_audit_log operator_notes_audit_log_changed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes_audit_log
    ADD CONSTRAINT operator_notes_audit_log_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES public.users(id);


--
-- Name: operator_notes_audit_log operator_notes_audit_log_note_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_notes_audit_log
    ADD CONSTRAINT operator_notes_audit_log_note_id_fkey FOREIGN KEY (note_id) REFERENCES public.operator_notes(id) ON DELETE CASCADE;


--
-- Name: operator_ownership_stats operator_ownership_stats_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_ownership_stats
    ADD CONSTRAINT operator_ownership_stats_server_id_fkey FOREIGN KEY (server_id) REFERENCES public.servers(id);


--
-- Name: operator_plans operator_plans_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.operator_plans
    ADD CONSTRAINT operator_plans_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: plan_group_members plan_group_members_operator_plan_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_group_members
    ADD CONSTRAINT plan_group_members_operator_plan_id_fkey FOREIGN KEY (operator_plan_id) REFERENCES public.operator_plans(id) ON DELETE CASCADE;


--
-- Name: plan_group_members plan_group_members_plan_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_group_members
    ADD CONSTRAINT plan_group_members_plan_group_id_fkey FOREIGN KEY (plan_group_id) REFERENCES public.plan_groups(id) ON DELETE CASCADE;


--
-- Name: plan_groups plan_groups_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plan_groups
    ADD CONSTRAINT plan_groups_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tier_list_favorites tier_list_favorites_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_favorites
    ADD CONSTRAINT tier_list_favorites_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: tier_list_favorites tier_list_favorites_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_favorites
    ADD CONSTRAINT tier_list_favorites_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tier_list_permissions tier_list_permissions_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_permissions
    ADD CONSTRAINT tier_list_permissions_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id);


--
-- Name: tier_list_permissions tier_list_permissions_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_permissions
    ADD CONSTRAINT tier_list_permissions_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: tier_list_permissions tier_list_permissions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_permissions
    ADD CONSTRAINT tier_list_permissions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: tier_list_stats tier_list_stats_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_stats
    ADD CONSTRAINT tier_list_stats_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: tier_list_versions tier_list_versions_published_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_versions
    ADD CONSTRAINT tier_list_versions_published_by_fkey FOREIGN KEY (published_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: tier_list_versions tier_list_versions_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_versions
    ADD CONSTRAINT tier_list_versions_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: tier_list_view_events tier_list_view_events_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_view_events
    ADD CONSTRAINT tier_list_view_events_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: tier_list_view_events tier_list_view_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_list_view_events
    ADD CONSTRAINT tier_list_view_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: tier_lists tier_lists_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_lists
    ADD CONSTRAINT tier_lists_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: tier_lists tier_lists_flair_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_lists
    ADD CONSTRAINT tier_lists_flair_id_fkey FOREIGN KEY (flair_id) REFERENCES public.tier_list_flairs(id) ON DELETE SET NULL;


--
-- Name: tier_placements tier_placements_tier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tier_placements
    ADD CONSTRAINT tier_placements_tier_id_fkey FOREIGN KEY (tier_id) REFERENCES public.tiers(id) ON DELETE CASCADE;


--
-- Name: tiers tiers_tier_list_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tiers
    ADD CONSTRAINT tiers_tier_list_id_fkey FOREIGN KEY (tier_list_id) REFERENCES public.tier_lists(id) ON DELETE CASCADE;


--
-- Name: user_building user_building_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_building
    ADD CONSTRAINT user_building_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_checkin user_checkin_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_checkin
    ADD CONSTRAINT user_checkin_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_enemy_progress user_enemy_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_enemy_progress
    ADD CONSTRAINT user_enemy_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_items user_items_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_items
    ADD CONSTRAINT user_items_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_medals user_medals_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_medals
    ADD CONSTRAINT user_medals_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_operator_modules user_operator_modules_user_id_operator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operator_modules
    ADD CONSTRAINT user_operator_modules_user_id_operator_id_fkey FOREIGN KEY (user_id, operator_id) REFERENCES public.user_operators(user_id, operator_id) ON DELETE CASCADE;


--
-- Name: user_operator_skills user_operator_skills_user_id_operator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operator_skills
    ADD CONSTRAINT user_operator_skills_user_id_operator_id_fkey FOREIGN KEY (user_id, operator_id) REFERENCES public.user_operators(user_id, operator_id) ON DELETE CASCADE;


--
-- Name: user_operators user_operators_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_operators
    ADD CONSTRAINT user_operators_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_roguelike_progress user_roguelike_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roguelike_progress
    ADD CONSTRAINT user_roguelike_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_sandbox_progress user_sandbox_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sandbox_progress
    ADD CONSTRAINT user_sandbox_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_scores user_scores_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scores
    ADD CONSTRAINT user_scores_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_settings user_settings_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_skins user_skins_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_skins
    ADD CONSTRAINT user_skins_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_stage_progress user_stage_progress_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_stage_progress
    ADD CONSTRAINT user_stage_progress_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_status user_status_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_status
    ADD CONSTRAINT user_status_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_support_units user_support_units_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_support_units
    ADD CONSTRAINT user_support_units_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: users users_server_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_server_id_fkey FOREIGN KEY (server_id) REFERENCES public.servers(id);


--
-- PostgreSQL database dump complete
--
