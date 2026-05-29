CREATE TABLE IF NOT EXISTS guild_auto_role (
    guild_id INTEGER PRIMARY KEY NOT NULL,
    auto_role_id INTEGER
);

CREATE TABLE IF NOT EXISTS guild_reaction_roles (
    guild_id INTEGER,
    channel_id  INTEGER NOT NULL,
    message_id  INTEGER NOT NULL,
    emoji       TEXT    NOT NULL,  -- "🟥" for unicode, "name:id" for custom
    role_id     INTEGER NOT NULL,
    PRIMARY KEY (message_id, emoji)
);

CREATE INDEX idx_reaction_roles_guild ON guild_reaction_roles(guild_id);

CREATE TABLE guild_asset_channel (
    guild_id   INTEGER PRIMARY KEY NOT NULL,
    channel_id INTEGER NOT NULL
);

CREATE TABLE guild_max_ping (
    guild_id INTEGER PRIMARY KEY NOT NULL,
    max_ping_per_message INTEGER NOT NULL,
    window_secs INTEGER, -- NULL = disabled
    window_max_pings INTEGER,
    action TEXT NOT NULL DEFAULT 'timeout', -- delete | warn | timeout | kick | ban
    timeout_secs INTEGER, -- only when action='timeout'
    exempt_role_id INTEGER -- members with this role bypass antispam
);
