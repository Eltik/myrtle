--
-- Tier list placements name an ENTITY, not only an operator.
--
-- A placement was keyed by (tier_id, operator_id). It becomes
-- (tier_id, entity_kind, entity_id), so a list can rank classes, enemies or
-- events next to operators without one id space colliding with another. Every
-- existing row is an operator and takes that kind from the column default.
--
-- `tier_lists.entity_kinds` is the set of kinds a list's editor offers. It is
-- what the pool shows, not a constraint on the rows: a list may hold kinds it
-- no longer offers. The allow-list of kinds lives in the backend
-- (`EntityKind`), so a new kind needs no migration.
--
-- Neither statement rewrites a row's data, so the placement and list update
-- triggers do not fire and every `updated_at` is unchanged.
--
ALTER TABLE tier_placements RENAME COLUMN operator_id TO entity_id;
ALTER TABLE tier_placements
    ADD COLUMN entity_kind varchar(24) NOT NULL DEFAULT 'operator';
ALTER TABLE tier_placements DROP CONSTRAINT tier_placements_pkey;
ALTER TABLE tier_placements
    ADD CONSTRAINT tier_placements_pkey PRIMARY KEY (tier_id, entity_kind, entity_id);

ALTER TABLE tier_lists
    ADD COLUMN entity_kinds text[] NOT NULL DEFAULT '{operator}'
    CONSTRAINT tier_lists_entity_kinds_nonempty CHECK (cardinality(entity_kinds) > 0);
