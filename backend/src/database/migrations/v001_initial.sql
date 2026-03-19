-- ═══════════════════════════════════════════════════════════════
-- USERS & AUTHENTICATION
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE servers (
    id          SMALLINT PRIMARY KEY,  -- 0=EN, 1=JP, 2=KR, 3=CN, 4=Bili, 5=TW
    code        VARCHAR(4) NOT NULL UNIQUE,
    name        VARCHAR(20) NOT NULL
);

CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uid         VARCHAR(20) NOT NULL,      -- Arknights UID
    server_id   SMALLINT NOT NULL REFERENCES servers(id),
    nickname    VARCHAR(50),
    level       SMALLINT DEFAULT 0,
    role        VARCHAR(20) NOT NULL DEFAULT 'user',
    avatar_id   VARCHAR(50),
    resume_id   VARCHAR(50),
    secretary   VARCHAR(50),               -- Assistant operator ID
    secretary_skin_id VARCHAR(50),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (uid, server_id)
);

-- Currencies, sanity, tickets, monthly card
CREATE TABLE user_status (
    user_id             UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    exp                 INT NOT NULL DEFAULT 0,
    orundum             INT NOT NULL DEFAULT 0,
    orundum_shard       INT NOT NULL DEFAULT 0,
    lmd                 INT NOT NULL DEFAULT 0,
    sanity              SMALLINT NOT NULL DEFAULT 0,
    max_sanity          SMALLINT NOT NULL DEFAULT 0,
    gacha_tickets       INT NOT NULL DEFAULT 0,
    ten_pull_tickets    INT NOT NULL DEFAULT 0,
    classic_gacha_tickets INT NOT NULL DEFAULT 0,
    classic_ten_pull_tickets INT NOT NULL DEFAULT 0,
    recruit_permits     INT NOT NULL DEFAULT 0,
    social_point        INT NOT NULL DEFAULT 0,
    hgg_shard           INT NOT NULL DEFAULT 0,
    lgg_shard           INT NOT NULL DEFAULT 0,
    practice_tickets    INT NOT NULL DEFAULT 0,
    gold                INT NOT NULL DEFAULT 0,
    monthly_sub_end     BIGINT,
    register_ts         BIGINT,
    last_online_ts      BIGINT,
    main_stage_progress VARCHAR(50),
    resume              TEXT,
    friend_num_limit    SMALLINT NOT NULL DEFAULT 0
);

CREATE TABLE user_settings (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    public_profile  BOOLEAN NOT NULL DEFAULT true,
    store_gacha     BOOLEAN NOT NULL DEFAULT false,
    share_stats     BOOLEAN NOT NULL DEFAULT false,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════════
-- USER ROSTER
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE user_operators (
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    operator_id     VARCHAR(50) NOT NULL,       -- e.g. "char_002_amiya"
    elite           SMALLINT NOT NULL DEFAULT 0, -- 0, 1, 2
    level           SMALLINT NOT NULL DEFAULT 1,
    exp             INT NOT NULL DEFAULT 0,
    potential       SMALLINT NOT NULL DEFAULT 0, -- 0-5
    skill_level     SMALLINT NOT NULL DEFAULT 1, -- 1-7
    favor_point     INT NOT NULL DEFAULT 0,
    skin_id         VARCHAR(50),
    default_skill   SMALLINT DEFAULT 0,
    voice_lan       VARCHAR(20),
    current_equip   VARCHAR(50),               -- Currently equipped module ID
    current_tmpl    VARCHAR(50),               -- Current template (Amiya etc)
    obtained_at     BIGINT,                    -- gain_time from game
    PRIMARY KEY (user_id, operator_id)
);

CREATE TABLE user_operator_skills (
    user_id         UUID NOT NULL,
    operator_id     VARCHAR(50) NOT NULL,
    skill_index     SMALLINT NOT NULL,           -- 0, 1, 2
    specialize_level SMALLINT NOT NULL DEFAULT 0, -- 0-3 (mastery)
    PRIMARY KEY (user_id, operator_id, skill_index),
    FOREIGN KEY (user_id, operator_id) REFERENCES user_operators(user_id, operator_id) ON DELETE CASCADE
);

CREATE TABLE user_operator_modules (
    user_id         UUID NOT NULL,
    operator_id     VARCHAR(50) NOT NULL,
    module_id       VARCHAR(50) NOT NULL,        -- e.g. "uniequip_002_amiya"
    module_level    SMALLINT NOT NULL DEFAULT 0,  -- 0-3
    locked          BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, operator_id, module_id),
    FOREIGN KEY (user_id, operator_id) REFERENCES user_operators(user_id, operator_id) ON DELETE CASCADE
);

-- ═══════════════════════════════════════════════════════════════
-- USER INVENTORY
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE user_items (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id     VARCHAR(50) NOT NULL,
    quantity    INT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, item_id)
);

-- ═══════════════════════════════════════════════════════════════
-- SKIN OWNERSHIP (Sparse — only owned skins)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE user_skins (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    skin_id     VARCHAR(50) NOT NULL,
    obtained_at BIGINT,
    PRIMARY KEY (user_id, skin_id)
);

-- ═══════════════════════════════════════════════════════════════
-- GAME STATE — JSONB for read-only data consumed by score calc
-- (Not queryable individually, just bulk-read per user)
-- ═══════════════════════════════════════════════════════════════

-- Stage completion: stage_id -> {state, complete_times, start_times, practice_times, ...}
CREATE TABLE user_stage_progress (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    stages      JSONB NOT NULL DEFAULT '{}'
);

-- Roguelike/IS progress per theme
CREATE TABLE user_roguelike_progress (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    theme_id    VARCHAR(20) NOT NULL,          -- rogue_1, rogue_2, ...rogue_5
    progress    JSONB NOT NULL DEFAULT '{}',   -- endings, buffs, relics, bp, challenges
    PRIMARY KEY (user_id, theme_id)
);

-- Sandbox/Reclamation Algorithm progress
CREATE TABLE user_sandbox_progress (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    progress    JSONB NOT NULL DEFAULT '{}'    -- places, nodes, choices, tech_trees, stories, logs
);

-- Medal completion
CREATE TABLE user_medals (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    medal_id    VARCHAR(50) NOT NULL,
    val         JSONB,                         -- completion value (number or array)
    first_ts    BIGINT,                        -- first obtain timestamp
    reach_ts    BIGINT,                        -- reach condition timestamp
    PRIMARY KEY (user_id, medal_id)
);

-- Building/base state snapshot (for base score calculation)
CREATE TABLE user_building (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    data        JSONB NOT NULL DEFAULT '{}'    -- Full building state
);

-- Check-in history
CREATE TABLE user_checkin (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    history     SMALLINT[] NOT NULL DEFAULT '{}'  -- Array of 0/1 per day
);

-- ═══════════════════════════════════════════════════════════════
-- SCORES
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE user_scores (
    user_id             UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    total_score         DOUBLE PRECISION NOT NULL DEFAULT 0,
    operator_score      DOUBLE PRECISION NOT NULL DEFAULT 0,
    stage_score         DOUBLE PRECISION NOT NULL DEFAULT 0,
    roguelike_score     DOUBLE PRECISION NOT NULL DEFAULT 0,
    sandbox_score       DOUBLE PRECISION NOT NULL DEFAULT 0,
    medal_score         DOUBLE PRECISION NOT NULL DEFAULT 0,
    base_score          DOUBLE PRECISION NOT NULL DEFAULT 0,
    skin_score          DOUBLE PRECISION NOT NULL DEFAULT 0,
    grade               VARCHAR(5),              -- S+, S, A, B, C, D
    composite_score     DOUBLE PRECISION NOT NULL DEFAULT 0,
    breakdown           JSONB,                   -- Full 170+ field breakdown for frontend
    calculated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ═══════════════════════════════════════════════════════════════
-- GACHA RECORDS
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE gacha_records (
    id              BIGSERIAL PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    char_id         VARCHAR(50) NOT NULL,
    pool_id         VARCHAR(50) NOT NULL,
    rarity          SMALLINT NOT NULL,
    pull_timestamp  BIGINT NOT NULL,
    pool_name       VARCHAR(100),
    gacha_type      VARCHAR(50),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, pull_timestamp, char_id, pool_id)
);

-- ═══════════════════════════════════════════════════════════════
-- TIER LISTS
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE tier_lists (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL UNIQUE,
    description     TEXT,
    list_type       VARCHAR(20) NOT NULL DEFAULT 'official',
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tiers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id    UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    name            VARCHAR(20) NOT NULL,
    display_order   SMALLINT NOT NULL,
    color           VARCHAR(7),
    description     TEXT,
    UNIQUE (tier_list_id, display_order)
);

CREATE TABLE tier_placements (
    tier_id         UUID NOT NULL REFERENCES tiers(id) ON DELETE CASCADE,
    operator_id     VARCHAR(50) NOT NULL,
    sub_order       SMALLINT NOT NULL DEFAULT 0,
    notes           TEXT,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tier_id, operator_id)
);

CREATE TABLE tier_list_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_list_id    UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    version         INT NOT NULL,
    snapshot        JSONB NOT NULL,              -- Full state at publication time
    changelog       TEXT,
    published_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tier_list_id, version)
);

CREATE TABLE tier_list_permissions (
    tier_list_id    UUID NOT NULL REFERENCES tier_lists(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission      VARCHAR(20) NOT NULL,        -- view, edit, publish, admin
    granted_by      UUID REFERENCES users(id),
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tier_list_id, user_id, permission)
);

-- ═══════════════════════════════════════════════════════════════
-- AUDIT LOG
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE audit_log (
    id              BIGSERIAL PRIMARY KEY,
    table_name      VARCHAR(50) NOT NULL,
    record_id       TEXT NOT NULL,               -- PK of the changed row
    action          VARCHAR(10) NOT NULL,        -- INSERT, UPDATE, DELETE
    old_data        JSONB,
    new_data        JSONB,
    changed_by      UUID,                        -- Set via session variable
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
