--
-- Squad Builder storage.
--
-- `user_squads`: the account's saved squads, four numbered slots. A squad is
-- read and written whole, so its twelve seats live in one `members` array
-- (`null` for an empty seat) and the support in `support`. Shape, operator
-- ids and build ranges are validated by the backend (`services::squad`), not
-- here.
--
CREATE TABLE user_squads (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    slot smallint NOT NULL CHECK (slot BETWEEN 1 AND 4),
    name varchar(40) NOT NULL,
    members jsonb NOT NULL,
    support jsonb,
    stage_id varchar(80),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (user_id, slot)
);

CREATE TRIGGER trg_user_squads_timestamp
    BEFORE UPDATE ON user_squads
    FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();

--
-- `user_game_squads`: the in-game squads as the last roster sync saw them,
-- read only. Each sync upserts the squads it carries, writing a row only when
-- its name or slots changed, and deletes the ones it no longer carries.
-- `slots` is twelve entries, `null` or `{"char_id", "skill_index",
-- "module_id"}`. `squad_index` has no range CHECK: the game's squad count is
-- in no table, so the sync payload decides it.
--
CREATE TABLE user_game_squads (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    squad_index smallint NOT NULL,
    name text NOT NULL,
    slots jsonb NOT NULL,
    synced_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (user_id, squad_index)
);
