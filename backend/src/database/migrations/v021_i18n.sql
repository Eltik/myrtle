--
-- Translation layer.
--
-- Keys and their English source text are extracted from the frontend at build
-- time and synced in here; the RENDERED text for every locale - English
-- included - lives in `ui_messages` and is editable at runtime by a Translator.
-- That split is deliberate: a copy fix must not require a frontend rebuild, and
-- the source text has to be in the database anyway for the editing UI to show
-- it, so the only question was which copy is authoritative. This one is.
--
-- Staleness is a comparison, not a state machine: a translation is stale where
-- `ui_messages.source_hash <> ui_message_keys.source_hash`. Nothing to set,
-- nothing to forget to clear.
--

CREATE TABLE IF NOT EXISTS locales (
    code             varchar(16) PRIMARY KEY,
    english_name     text NOT NULL,
    native_name      text NOT NULL,
    fallback_locale  varchar(16) REFERENCES locales(code) ON DELETE SET NULL,
    -- Which game client's data serves this locale's operator/skill/stage text.
    -- Each Arknights region ships its own translation, so picking the region
    -- does almost all of the game-data work; `gamedata_overrides` covers gaps.
    gamedata_server  varchar(8) NOT NULL DEFAULT 'en',
    enabled          boolean NOT NULL DEFAULT false,
    sort_order       integer NOT NULL DEFAULT 100,
    created_at       timestamptz NOT NULL DEFAULT now()
);

-- English is the source locale and is always enabled. It has no fallback: the
-- bundled source catalog is the floor beneath it.
INSERT INTO locales (code, english_name, native_name, gamedata_server, enabled, sort_order)
VALUES ('en', 'English', 'English', 'en', true, 0)
ON CONFLICT (code) DO NOTHING;

CREATE TABLE IF NOT EXISTS ui_message_keys (
    key            text PRIMARY KEY,
    namespace      text NOT NULL,
    source_text    text NOT NULL,
    source_hash    varchar(16) NOT NULL,
    description    text,
    -- Declared ICU placeholders, e.g. ["count"]. A translation that drops one
    -- or invents one is rejected at write time; this column is what makes that
    -- check possible without parsing the source on every save.
    placeholders   jsonb NOT NULL DEFAULT '[]'::jsonb,
    first_seen_at  timestamptz NOT NULL DEFAULT now(),
    last_seen_at   timestamptz NOT NULL DEFAULT now(),
    -- Sync marks vanished keys inactive rather than deleting them, so a key
    -- that comes back (a reverted refactor) keeps its translations.
    is_active      boolean NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_ui_message_keys_namespace ON ui_message_keys(namespace) WHERE is_active;

CREATE TABLE IF NOT EXISTS ui_messages (
    key          text NOT NULL REFERENCES ui_message_keys(key) ON DELETE CASCADE,
    locale       varchar(16) NOT NULL REFERENCES locales(code) ON DELETE CASCADE,
    value        text NOT NULL,
    -- The source_hash this value was translated against. Compare to the key's
    -- current source_hash to find translations the English has moved out from
    -- under.
    source_hash  varchar(16) NOT NULL,
    updated_by   uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (key, locale)
);

CREATE INDEX IF NOT EXISTS idx_ui_messages_locale ON ui_messages(locale);

CREATE TABLE IF NOT EXISTS ui_message_audit_log (
    id           bigserial PRIMARY KEY,
    message_key  text NOT NULL,
    locale       varchar(16) NOT NULL,
    old_value    text,
    new_value    text,
    changed_by   uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    changed_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_ui_message_audit_changed_at ON ui_message_audit_log(changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_ui_message_audit_key ON ui_message_audit_log(message_key, locale, changed_at DESC);

--
-- Long-form prose (terms, privacy) translated as whole documents rather than
-- as keys. Splitting a legal document into 200 disconnected string keys
-- produces nonsense in any language, so these are markdown blobs edited in the
-- same MarkdownEditor the operator-notes screen uses.
--
CREATE TABLE IF NOT EXISTS ui_documents (
    slug        text NOT NULL,
    locale      varchar(16) NOT NULL REFERENCES locales(code) ON DELETE CASCADE,
    title       text NOT NULL,
    body        text NOT NULL,
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (slug, locale)
);

--
-- Per-locale grants. Shape-identical to tier_list_permissions on purpose: the
-- `translator` global role is only a ticket into the admin panel, and THIS
-- table is the authority on which locales a given user may write. Recruiting a
-- Korean translator is one row, not a release.
--
-- Because the check is a database read either way, the stale-JWT problem that
-- affects `users.role` does not apply here: a grant takes effect on the next
-- request.
--
CREATE TABLE IF NOT EXISTS translation_permissions (
    locale      varchar(16) NOT NULL REFERENCES locales(code) ON DELETE CASCADE,
    user_id     uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission  varchar(20) NOT NULL,
    granted_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    granted_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (locale, user_id, permission)
);

CREATE INDEX IF NOT EXISTS idx_translation_permissions_user ON translation_permissions(user_id);

--
-- Game-data text overrides. Generalises release_overrides.en_name from one
-- field on one kind to any field on any kind, for any locale. Applied by the
-- backend at serialisation time so that every consumer - the site, the Discord
-- bot, the API - sees one answer.
--
CREATE TABLE IF NOT EXISTS gamedata_overrides (
    locale      varchar(16) NOT NULL REFERENCES locales(code) ON DELETE CASCADE,
    kind        varchar(32) NOT NULL,
    entity_id   text NOT NULL,
    field       varchar(64) NOT NULL,
    value       text NOT NULL,
    updated_by  uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (locale, kind, entity_id, field)
);

CREATE INDEX IF NOT EXISTS idx_gamedata_overrides_locale_kind ON gamedata_overrides(locale, kind);
