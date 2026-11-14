--
-- OAuth 2.0 provider: "Sign in with myrtle.moe".
--
-- A third-party site registers an application, sends a player to the consent
-- page, and exchanges the code it gets back for a token that reads that
-- player's identity (`GET /api/oauth/userinfo`). The flow is the
-- authorization-code grant of RFC 6749 with PKCE (RFC 7636).
--
-- No secret is stored in the clear. A client secret, an authorization code, an
-- access token and a refresh token are each 32 random bytes; what a row holds
-- is the SHA-256 of one, which is enough to check a presented value and
-- useless for presenting one. A fast hash is right here: the input is not a
-- password someone chose, so there is nothing to brute-force.
--
-- Every table hangs off `users` or `oauth_apps` with ON DELETE CASCADE, so
-- deleting an account removes the apps it registered, the access it granted
-- and every token under either, and deleting an app signs everyone out of it.
--

--
-- `oauth_apps`: one registered application. `id` is the public `client_id`.
-- `redirect_uris` are compared to the request's `redirect_uri` as exact
-- strings; their shape is validated by the backend (`core::auth::oauth`), not
-- here.
--
CREATE TABLE oauth_apps (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name varchar(64) NOT NULL,
    description varchar(280) NOT NULL DEFAULT '',
    redirect_uris text[] NOT NULL,
    secret_hash bytea NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_oauth_apps_owner ON oauth_apps (owner_id);

CREATE TRIGGER trg_oauth_apps_timestamp
    BEFORE UPDATE ON oauth_apps
    FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();

--
-- `oauth_grants`: a player's standing consent to one app, one row per pair.
-- `scope` is the space-separated set they last approved. Revoking access
-- deletes the row, and the codes and tokens under it go with it.
--
CREATE TABLE oauth_grants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    app_id uuid NOT NULL REFERENCES oauth_apps(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scope text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone,
    UNIQUE (app_id, user_id)
);

CREATE INDEX idx_oauth_grants_user ON oauth_grants (user_id);

--
-- `oauth_codes`: authorization codes, each good for one exchange within five
-- minutes. A spent code keeps its row (`consumed_at` set) until it expires, so
-- a second exchange of the same code can be told from an unknown one and
-- answered by revoking what the first exchange issued.
--
CREATE TABLE oauth_codes (
    code_hash bytea PRIMARY KEY,
    grant_id uuid NOT NULL REFERENCES oauth_grants(id) ON DELETE CASCADE,
    redirect_uri text NOT NULL,
    scope text NOT NULL,
    code_challenge text,
    expires_at timestamp with time zone NOT NULL,
    consumed_at timestamp with time zone
);

CREATE INDEX idx_oauth_codes_expiry ON oauth_codes (expires_at);

--
-- `oauth_tokens`: one row per sign-in, holding the current access and refresh
-- token of that session. A refresh replaces both hashes in place, which is
-- what makes the old refresh token stop working.
--
CREATE TABLE oauth_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    grant_id uuid NOT NULL REFERENCES oauth_grants(id) ON DELETE CASCADE,
    access_hash bytea NOT NULL UNIQUE,
    refresh_hash bytea NOT NULL UNIQUE,
    scope text NOT NULL,
    access_expires_at timestamp with time zone NOT NULL,
    refresh_expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_oauth_tokens_grant ON oauth_tokens (grant_id, created_at);
CREATE INDEX idx_oauth_tokens_expiry ON oauth_tokens (refresh_expires_at);
