--
-- Music library: the Monster Siren Records catalogue cache and the account's
-- likes, listening history and playlists.
--
-- `siren_albums` / `siren_songs` mirror the public Monster Siren API
-- (monster-siren.hypergryph.com). The sync job (`services::music::siren`)
-- refreshes them on an interval and fetches each album's and song's detail
-- ONCE, so a restart never re-crawls the label's servers. `list_order` is the
-- album's position in the API's list (0 = newest), `track_order` the song's
-- position in its album detail. `lyric_text` is the LRC file, fetched the
-- first time a reader asks for it.
--
CREATE TABLE siren_albums (
    cid text PRIMARY KEY,
    name text NOT NULL,
    intro text NOT NULL DEFAULT '',
    belong text NOT NULL DEFAULT '',
    cover_url text NOT NULL,
    cover_de_url text,
    artists text[] NOT NULL DEFAULT '{}',
    list_order integer NOT NULL,
    fetched_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE siren_songs (
    cid text PRIMARY KEY,
    album_cid text NOT NULL REFERENCES siren_albums(cid) ON DELETE CASCADE,
    name text NOT NULL,
    artists text[] NOT NULL DEFAULT '{}',
    lyric_url text,
    mv_url text,
    mv_cover_url text,
    track_order integer NOT NULL DEFAULT 0,
    lyric_text text,
    fetched_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_siren_songs_album ON siren_songs (album_cid, track_order);

--
-- Account data. A track is named by a prefixed id, `siren:<cid>` for a label
-- release or `game:<track id>` for an in-game track, so one like, one play and
-- one playlist row shape serves both collections. The backend checks the
-- prefix (`services::music::library`); existence is not checked, because a
-- track the catalogue later drops must not take the row with it.
--
CREATE TABLE music_likes (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    track_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (user_id, track_id)
);

CREATE TABLE music_history (
    id bigserial PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    track_id text NOT NULL,
    listened_seconds real NOT NULL DEFAULT 0,
    played_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_music_history_user_time ON music_history (user_id, played_at DESC);

CREATE TABLE music_playlists (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name varchar(60) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_music_playlists_user ON music_playlists (user_id, created_at);

CREATE TRIGGER trg_music_playlists_timestamp
    BEFORE UPDATE ON music_playlists
    FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();

CREATE TABLE music_playlist_tracks (
    playlist_id uuid NOT NULL REFERENCES music_playlists(id) ON DELETE CASCADE,
    track_id text NOT NULL,
    position integer NOT NULL,
    added_at timestamp with time zone DEFAULT now() NOT NULL,
    PRIMARY KEY (playlist_id, track_id)
);
