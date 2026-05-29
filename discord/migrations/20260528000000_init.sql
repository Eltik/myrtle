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
