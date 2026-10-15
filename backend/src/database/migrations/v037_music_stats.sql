--
-- Community music stats: how often each track is listened to and downloaded,
-- by everyone, signed in or not.
--
-- `music_stat_events` is the record: one row per COUNTED listen or download,
-- timestamped, so any window (this week, this month, a calendar year for a
-- yearly recap) is an aggregate over it. A signed-in caller's rows carry
-- their `user_id`; an anonymous caller's carry nothing that identifies them.
-- Deleting an account keeps its rows as anonymous ones, so the community
-- counts do not shrink.
--
-- `music_track_stats` holds the all-time totals per track, raised in the same
-- transaction as each event, so the whole catalogue's counters are one
-- primary-key scan of about a thousand rows.
--
-- `music_stat_dedupe` decides whether a call counts: one row per (kind,
-- client, track) holding when that client last COUNTED for that track. A
-- call counts when there is no row, or the row is older than the window
-- (`services::music::stats`: 30 minutes for a listen, 24 hours for a
-- download). `client` is `u:<user uuid>` for a signed-in caller and
-- `a:<hex>` for an anonymous one, the hex being an HMAC of the caller's
-- address (an IPv6 address cut to its /64) under a server secret; the raw
-- address is never stored. Rows past the longest window decide nothing and
-- are swept, so the table holds about a day of clients. UNLOGGED: a crash
-- empties it, which at worst lets each client count once more.
--
CREATE TABLE music_stat_events (
    id bigserial PRIMARY KEY,
    kind text NOT NULL CHECK (kind IN ('listen', 'download')),
    track_id text NOT NULL,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

-- Top tracks and top listeners over a window: an index-only range scan.
CREATE INDEX idx_music_stat_events_listen_time ON music_stat_events (created_at)
    INCLUDE (track_id, user_id) WHERE kind = 'listen';
-- One track's listeners over a window.
CREATE INDEX idx_music_stat_events_track_listen ON music_stat_events (track_id, created_at)
    INCLUDE (user_id) WHERE kind = 'listen' AND user_id IS NOT NULL;
-- One account's events in time order: its listener rank, a yearly recap.
CREATE INDEX idx_music_stat_events_user_time ON music_stat_events (user_id, created_at)
    INCLUDE (kind, track_id) WHERE user_id IS NOT NULL;

CREATE TABLE music_track_stats (
    track_id text PRIMARY KEY,
    listens bigint NOT NULL DEFAULT 0,
    downloads bigint NOT NULL DEFAULT 0,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE UNLOGGED TABLE music_stat_dedupe (
    kind text NOT NULL,
    client text NOT NULL,
    track_id text NOT NULL,
    counted_at timestamp with time zone NOT NULL,
    PRIMARY KEY (kind, client, track_id)
);

CREATE INDEX idx_music_stat_dedupe_counted ON music_stat_dedupe (counted_at);

-- Listening time over a window comes from the plays themselves.
CREATE INDEX idx_music_history_time ON music_history (played_at)
    INCLUDE (user_id, track_id, listened_seconds);
CREATE INDEX idx_music_history_track_time ON music_history (track_id, played_at)
    INCLUDE (user_id, listened_seconds);
