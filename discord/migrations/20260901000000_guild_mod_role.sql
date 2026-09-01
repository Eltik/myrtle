-- Per-guild moderator role.
--
-- Holding this role is an ALTERNATIVE to holding the Discord permission an elevated command
-- asks for, never a replacement: members who already have the permission keep it. Stored as
-- its own table rather than a column on an existing one because it has no natural owner —
-- it gates every elevated command, not one feature.
--
-- The row is deleted by `/modrole remove`, so "no row" and "no mod role" are the same state.
CREATE TABLE IF NOT EXISTS guild_mod_role (
    guild_id    INTEGER PRIMARY KEY NOT NULL,
    mod_role_id INTEGER NOT NULL
);
