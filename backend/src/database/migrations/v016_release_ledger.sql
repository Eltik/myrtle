--
-- CN -> EN release mapping, layer 1 and the override table.
-- Design and the measurements behind it: docs/release/CN_EN_RELEASE_MAPPING.md.
--
-- gamedata_sightings is the first-seen LEDGER: one row per (server, kind, id)
-- for every activity, gacha pool, skin, skin shop window, retro act and
-- character the typed gamedata tables carry, written by diffing the tables
-- against this table on every load (boot and hot reload). The live tables lose
-- history on every patch; this keeps it. first_seen_at never changes after the
-- insert. start_time / end_time / last_seen_at are refreshed on every pass so a
-- date that MOVED in a later patch is visible rather than silently overwritten
-- in memory only.
--
-- server_id follows Server::index() (EN 0, JP 1, KR 2, CN 3, Bilibili 4, TW 5),
-- the same convention as the ownership aggregates.
--
-- release_overrides is the manual layer of the precedence
--   EN sighting > override > estimate.
-- A row is keyed on the CN id so it can be written before the EN id exists. It
-- is never deleted by the ledger when the EN sighting lands; the sighting wins
-- at read time and the row stays for audit.
--

CREATE TABLE public.gamedata_sightings (
    server_id smallint NOT NULL,
    kind text NOT NULL,
    id text NOT NULL,
    first_seen_at timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    res_version text,
    start_time bigint,
    end_time bigint,
    PRIMARY KEY (server_id, kind, id)
);

CREATE INDEX gamedata_sightings_first_seen_idx
    ON public.gamedata_sightings (server_id, kind, first_seen_at);

CREATE TABLE public.release_overrides (
    kind text NOT NULL,
    cn_id text NOT NULL,
    en_id text,
    en_start bigint,
    en_end bigint,
    source text NOT NULL DEFAULT 'manual',
    note text NOT NULL DEFAULT '',
    updated_by uuid,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (kind, cn_id)
);
