-- Durable Yostar credentials, so a game session outlives its cache entry.
--
-- `CacheKey::GameSession` holds a live session for one hour. The minted
-- `secret` inside it is short-lived and re-derivable, but the durable
-- `yostar_uid` / `yostar_token` pair it carries is NOT re-derivable without a
-- fresh email code. Keeping that pair only in Redis meant every cache expiry,
-- eviction, or backend restart forced a full re-login before the user could
-- re-sync -- the site token lasts 7 days, the game session lasted 1 hour.
-- This table is where the durable pair actually lives; the cache keeps its
-- real job, a hot copy of the live session.
--
-- The pair is credential material, not a cache, and is stored sealed:
-- XChaCha20-Poly1305 under `GAME_CREDENTIAL_KEY`, with the row's `user_id` as
-- additional authenticated data so a ciphertext cannot be moved between rows.
-- A database dump alone therefore yields no usable game credentials.
--
-- Deleting the row is the user-facing "disconnect" action: it revokes our
-- ability to reach the account until the user supplies another email code.
-- ON DELETE CASCADE means account deletion revokes it too.
CREATE TABLE IF NOT EXISTS user_game_credentials (
    user_id    UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    nonce      BYTEA NOT NULL,
    ciphertext BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
