--
-- A grid names the entity kinds its cells may hold.
--
-- `grids.entity_kinds` is the set of kinds the grid's picker offers, and,
-- unlike `tier_lists.entity_kinds`, a constraint the backend enforces on
-- every save: each cell's pick must be of an allowed kind. A fork copies the
-- set and cannot change it (`services::grid`).
--
-- Every existing grid was made when any kind could be picked, so it takes
-- every kind known at this migration, which is also the column default, as
-- `tier_lists.entity_kinds` keeps its own. The allow-list of kinds lives in
-- the backend (`EntityKind`), so a new kind needs no migration.
--
-- Adding a column with a constant default rewrites no row, so the
-- `updated_at` trigger does not fire and every `updated_at` is unchanged.
--
ALTER TABLE grids
    ADD COLUMN entity_kinds text[] NOT NULL
    DEFAULT '{operator,class,subclass,enemy,event,faction,stronghold_bond,skin,module,skill,integrated_strategies,story_sprite,main_story}'
    CONSTRAINT grids_entity_kinds_nonempty CHECK (cardinality(entity_kinds) > 0);
