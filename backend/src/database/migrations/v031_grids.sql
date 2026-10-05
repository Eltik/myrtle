--
-- Grids: a titled R x C board whose cells each carry a free-text label and,
-- optionally, one tier-list entity.
--
-- A grid is read and written whole, so its cells live in one `cells` array,
-- row-major, as `{"label", "entity_kind", "entity_id"}`. Shape, lengths and
-- entity refs are validated by the backend (`services::grid`), not here; the
-- table only pins the size bounds.
--
-- `template_of` points at the grid this one was forked from. Deleting the
-- source keeps the fork and clears the link.
--
CREATE TABLE grids (
    id uuid DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    slug varchar(120) NOT NULL UNIQUE,
    title varchar(100) NOT NULL,
    description text,
    created_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rows smallint NOT NULL CHECK (rows BETWEEN 1 AND 10),
    cols smallint NOT NULL CHECK (cols BETWEEN 1 AND 10),
    cells jsonb NOT NULL,
    is_listed boolean DEFAULT true NOT NULL,
    template_of uuid REFERENCES grids(id) ON DELETE SET NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_grids_created_by ON grids (created_by);
CREATE INDEX idx_grids_template_of ON grids (template_of);
CREATE INDEX idx_grids_listed_updated ON grids (is_listed, updated_at DESC);

CREATE TRIGGER trg_grids_timestamp
    BEFORE UPDATE ON grids
    FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();
