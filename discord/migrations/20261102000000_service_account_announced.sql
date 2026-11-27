-- The last state of each backend service account the bot announced.
--
-- server is the backend's server code (en, bili, ...). state is 'ok' or 'refused', never
-- 'unknown': a backend that has not used the account yet leaves the row as it was, so a backend
-- restart in the middle of an outage neither repeats the outage post nor fakes a recovery.
-- refused_since is the outage start the backend reported, kept for the recovery post.
CREATE TABLE IF NOT EXISTS service_account_announced (
    server        TEXT PRIMARY KEY NOT NULL,
    state         TEXT NOT NULL,
    refused_since INTEGER,
    updated_at    INTEGER NOT NULL
);
