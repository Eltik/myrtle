# Sign in with myrtle.moe

[myrtle.moe](https://myrtle.moe) is an OAuth 2.0 provider. Another site sends a player to myrtle.moe, the player approves, and the site learns who they are: a stable account id, their in-game nickname, their game UID and server, and their avatar. Nothing else is shared and nothing can be changed. Served by the [Myrtle backend](README.md), with the consent screen in the [frontend](../frontend/README.md).

[![OAuth 2.0](https://img.shields.io/badge/OAuth-2.0-blue)](https://datatracker.ietf.org/doc/html/rfc6749)
[![PKCE](https://img.shields.io/badge/PKCE-S256-blue)](https://datatracker.ietf.org/doc/html/rfc7636)
[![Revocation](https://img.shields.io/badge/Revocation-RFC_7009-blue)](https://datatracker.ietf.org/doc/html/rfc7009)
[![Backend CI](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml)

It is the standard authorization-code grant ([RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749)) with optional PKCE ([RFC 7636](https://datatracker.ietf.org/doc/html/rfc7636)), so any OAuth 2.0 client library works. A complete client in one dependency-free file is in [`examples/oauth-client`](../examples/oauth-client/README.md).

## Features

- **One scope, read-only** - `identify` returns the player's account id, nickname, UID, server, and avatar. An access token opens nothing else.
- **Authorization code + PKCE** - `S256` challenges are supported and recommended. `plain` is refused.
- **Confidential clients only** - every application has a secret and every token request sends it. There is no public-client or PKCE-only flow, so the exchange belongs on your server.
- **Rotating refresh tokens** - each refresh returns a new pair and retires the old one.
- **Revocation** - end a sign-in from your side ([RFC 7009](https://datatracker.ietf.org/doc/html/rfc7009)), or let the player remove your application in Settings.
- **Opaque tokens** - 32 random bytes behind a `myrtle_` prefix, stored only as SHA-256 hashes.
- **Consent every time** - the player sees your application's name and what it will learn on every sign-in.

There is no discovery document (`/.well-known/...`), no introspection endpoint, and no client-credentials, device, or implicit grant.

## Endpoints

| Endpoint      | Request                                         |
|---------------|-------------------------------------------------|
| Authorization | `GET https://myrtle.moe/oauth/authorize`        |
| Token         | `POST https://api.myrtle.moe/api/oauth/token`   |
| Revocation    | `POST https://api.myrtle.moe/api/oauth/revoke`  |
| User info     | `GET https://api.myrtle.moe/api/oauth/userinfo` |

The authorization endpoint is on the website. The other three are on the API host.

## Quick Start

### 1. Register an application

Sign in to myrtle.moe, open **Settings**, then **Applications**, and create an application under "Your applications".

| Field         | Rules                                                                                     |
|---------------|-------------------------------------------------------------------------------------------|
| Name          | Required, up to 64 characters. Shown to players on the consent screen with your in-game name. |
| Description   | Optional, up to 280 characters, one line. Shown on the consent screen.                    |
| Redirect URIs | 1 to 10. Where a player is sent back to.                                                  |

A redirect URI must be an absolute `https` URL of at most 512 bytes, with no fragment and no `user:password@` part. A query string is fine. `http` is accepted only when the host is exactly `localhost`, `127.0.0.1`, or `[::1]`, for development. Custom schemes such as `myapp://` are refused.

You get a **client ID** (a UUID) and a **client secret** (`myrtle_cs_...`). The secret is shown once and stored only as a hash. If you lose it, "New secret" replaces it: the old one stops working at once, and people who already signed in stay signed in. Keep the secret on your server. It does not belong in a browser or a mobile app.

An account can register up to 10 applications.

### 2. Send the player to myrtle.moe

Redirect the player's browser to the authorization endpoint:

```text
https://myrtle.moe/oauth/authorize
    ?response_type=code
    &client_id=YOUR_CLIENT_ID
    &redirect_uri=https%3A%2F%2Fyour.site%2Fauth%2Fmyrtle%2Fcallback
    &scope=identify
    &state=RANDOM_VALUE_YOU_STORED
    &code_challenge=BASE64URL_SHA256_OF_YOUR_VERIFIER
    &code_challenge_method=S256
```

| Parameter               | Required         | Description                                                                                        |
|-------------------------|------------------|----------------------------------------------------------------------------------------------------|
| `response_type`         | Yes              | `code`.                                                                                            |
| `client_id`             | Yes              | Your application's client ID.                                                                      |
| `redirect_uri`          | Yes              | One of your registered URIs, character for character. May be left out only if you registered exactly one. |
| `scope`                 | No               | `identify` is the only scope and the default. Names are case-sensitive.                            |
| `state`                 | Recommended      | A random value you tie to the player's browser session and check when they come back. Up to 1024 bytes, returned unchanged. |
| `code_challenge`        | Recommended      | PKCE. The unpadded base64url SHA-256 of a random `code_verifier` that you keep: 43 characters.     |
| `code_challenge_method` | With a challenge | `S256`. `plain` is not accepted.                                                                   |

The `code_verifier` is 43 to 128 characters from `A-Z a-z 0-9 - . _ ~`.

The player signs in if they are not already, sees your application's name and what it will learn, and chooses Authorize or Cancel. They are asked every time, including when they have authorized you before.

Three problems are shown to the player on myrtle.moe, with no redirect: a missing or unknown `client_id`, a `redirect_uri` that is not registered, and an omitted `redirect_uri` when you registered more than one. Every other problem is reported to your redirect URI, as below.

### 3. Handle the redirect back

The player returns to your `redirect_uri` with either a code:

```text
https://your.site/auth/myrtle/callback?code=myrtle_ac_...&state=RANDOM_VALUE_YOU_STORED
```

or an error (RFC 6749 section 4.1.2.1):

```text
https://your.site/auth/myrtle/callback?error=access_denied&error_description=...&state=...
```

| `error`                     | Meaning                                                              |
|-----------------------------|----------------------------------------------------------------------|
| `access_denied`             | The player pressed Cancel.                                           |
| `invalid_request`           | A parameter is missing or malformed. `error_description` says which. |
| `unsupported_response_type` | `response_type` was not `code`.                                      |
| `invalid_scope`             | An unknown scope was requested.                                      |

The parameters are appended to whatever query string your redirect URI already has. A `state` longer than 1024 bytes is answered with `invalid_request` and is **not** echoed back.

Check that `state` is the value you stored for this browser before doing anything else. If it is not, stop: someone may be trying to complete a sign-in they started elsewhere.

### 4. Exchange the code for tokens

From your server, within five minutes:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/token \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d grant_type=authorization_code \
  -d code="$CODE" \
  -d redirect_uri=https://your.site/auth/myrtle/callback \
  -d code_verifier="$CODE_VERIFIER"
```

The body is `application/x-www-form-urlencoded`, each parameter at most once. JSON is refused. Authenticate with HTTP Basic (`client_id` as the username, `client_secret` as the password, each form-encoded), or send `client_id` and `client_secret` as two more form fields. If you send both, the Basic header wins, and a different `client_id` in the body is an `invalid_request`.

```json
{
  "access_token": "myrtle_at_...",
  "token_type": "Bearer",
  "expires_in": 604800,
  "refresh_token": "myrtle_rt_...",
  "scope": "identify"
}
```

- A code works **once**, and a failed exchange spends it too. Once your client has authenticated, a wrong `redirect_uri` or a wrong or missing `code_verifier` burns the code; start the sign-in again.
- Presenting a spent code a second time fails, and also signs that player out of your application everywhere: every token they hold for it is revoked, because a code seen twice may have leaked.
- `code_verifier` is required if the authorization request carried a `code_challenge` (`invalid_request` without it), and refused if it did not (`invalid_grant`).
- `redirect_uri`, when sent, must be the one the code was issued for.

### 5. Read the player's identity

```bash
curl https://api.myrtle.moe/api/oauth/userinfo \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

```json
{
  "id": "05990f3d-bf6b-4561-88cd-d93fe58f9b40",
  "username": "Doctor",
  "tag": "1234",
  "uid": "12345678",
  "server": "en",
  "avatar_id": "char_002_amiya",
  "avatar_url": "https://api.myrtle.moe/api/avatar/char_002_amiya"
}
```

| Field        | Description                                                                                                  |
|--------------|--------------------------------------------------------------------------------------------------------------|
| `id`         | The myrtle.moe account id. Stable for the life of the account. **Key your connected account on this.**       |
| `username`   | The player's in-game nickname. Not unique, and it changes when they rename in game. For an account whose nickname has not synced yet it is the game UID. |
| `tag`        | The digits after the `#` in the in-game name, or `null`.                                                     |
| `uid`        | The in-game UID on `server`. `https://myrtle.moe/user/{uid}` is the player's profile page; it opens only if they made their profile public. |
| `server`     | `en`, `jp`, `kr`, `cn`, `bili`, or `tw`.                                                                     |
| `avatar_id`  | The avatar the player picked in game, or `null` if they never set one.                                       |
| `avatar_url` | An image of that avatar (Amiya's when `avatar_id` is `null`). Safe to hotlink.                               |

## Token Lifetimes

| Token              | Format          | Lifetime                              |
|--------------------|-----------------|---------------------------------------|
| Authorization code | `myrtle_ac_...` | 5 minutes, single use                 |
| Access token       | `myrtle_at_...` | 7 days                                |
| Refresh token      | `myrtle_rt_...` | 90 days from its last use, single use |

Tokens are opaque. Do not parse them.

If all you need is to know who signed in, call `userinfo` once and discard the tokens. To keep a connection alive (for example to refresh a displayed nickname), store the refresh token and trade it for a new pair when the access token expires or `userinfo` answers 401:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/token \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d grant_type=refresh_token \
  -d refresh_token="$REFRESH_TOKEN"
```

The response has the same shape as before. **Every refresh returns a new refresh token and retires the one you sent**, so store the new one. The old access token stops working too. A retired refresh token answers `invalid_grant` and nothing else happens; the current pair stays valid.

A `scope` field on a refresh is checked for unknown names and otherwise ignored. The token keeps the scope it was granted.

Each sign-in is its own session with its own token pair. A player can hold up to 20 sessions with one application; the 21st sign-in ends the oldest.

## Signing Out

To end a sign-in from your side, revoke either of its tokens ([RFC 7009](https://datatracker.ietf.org/doc/html/rfc7009)). Both stop working:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/revoke \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d token="$REFRESH_TOKEN"
```

The answer is `200` with an empty body, also for a token that was already dead or that belongs to another application (which is left untouched). `token_type_hint` is not needed and is ignored.

A player can remove your application's access themselves, in Settings under Applications, "Authorized applications". Your tokens for them then answer `invalid_token` / `invalid_grant`; treat that as signed out.

## Errors

The token, revocation, and userinfo endpoints answer errors in the OAuth format (RFC 6749 section 5.2), not in the envelope the rest of the API uses. Every answer the endpoints give, success or error, carries `Cache-Control: no-store`; a `429` from the rate limiter in front of them does not.

```json
{ "error": "invalid_grant", "error_description": "the authorization code is invalid, expired or already used" }
```

| Status | `error`                   | Meaning                                                                                              |
|--------|---------------------------|------------------------------------------------------------------------------------------------------|
| 400    | `invalid_request`         | A parameter is missing or repeated, or the body is not form-encoded.                                 |
| 400    | `invalid_grant`           | The code or refresh token is invalid, expired, already used, or belongs to another application; or the `redirect_uri` or `code_verifier` does not match. |
| 400    | `unsupported_grant_type`  | `grant_type` is neither `authorization_code` nor `refresh_token`.                                    |
| 400    | `invalid_scope`           | An unknown scope.                                                                                    |
| 401    | `invalid_client`          | Missing credentials, unknown `client_id`, or wrong `client_secret`. Sent with `WWW-Authenticate: Basic realm="myrtle.moe"`. |
| 401    | `invalid_token`           | (`userinfo`) The access token is missing, expired, or revoked. Sent with `WWW-Authenticate: Bearer error="invalid_token"`. |
| 403    | `insufficient_scope`      | (`userinfo`) The token was not granted `identify`.                                                   |
| 429    | -                         | Rate limited. See below.                                                                             |
| 500    | `server_error`            | Retry.                                                                                               |
| 503    | `temporarily_unavailable` | The server is busy. Retry shortly.                                                                   |

The body is parsed first: one that is not form-encoded, or that repeats a parameter, is `invalid_request` even with no credentials. After that the client is authenticated before any grant parameter is read, so a well-formed request with missing or wrong credentials is always `invalid_client`.

### Rate limits

OAuth requests share the API's default budget: 100 requests per minute per IP address in the shipped configuration (`rate_limit.rpm`), counted together with every other default-bucket request from that address. A request over the limit answers `429` in the API's usual envelope, not the OAuth format, with `Retry-After: 60`:

```json
{ "error": { "code": "RATE_LIMITED", "message": "rate limited" } }
```

## What an Access Token Can and Cannot Do

An access token is accepted by `GET /api/oauth/userinfo` and nowhere else. It is not the session token the website uses: it cannot read a private profile, change settings, trigger a roster sync, or delete an account. Every other route that requires sign-in answers it with `401 UNAUTHORIZED`, and routes where sign-in is optional treat it as anonymous. Likewise, a website session token and a refresh token are not accepted by `userinfo`.

## Internals

The rest of this page is for people working on the provider, not for people building against it.

### Site endpoints

The consent page and the Applications settings panel call these with the website's session token. They answer errors in the API's usual envelope, and a third-party application never calls them.

| Request                                     | Purpose                                                            |
|---------------------------------------------|--------------------------------------------------------------------|
| `GET /api/oauth/authorize`                  | Validate an authorization request. Answers what the consent page shows (`status: "consent"`), or where to send the player (`status: "redirect"`). |
| `POST /api/oauth/authorize`                 | Record the player's decision (`approve`) and answer `redirect_to`. |
| `GET /api/oauth/authorizations`             | The applications the player has authorized.                        |
| `DELETE /api/oauth/authorizations/{app_id}` | Remove an application's access. 204, or 404 with no grant.         |
| `GET /api/oauth/apps`                       | The caller's own applications.                                     |
| `POST /api/oauth/apps`                      | Register an application. 201 with the `client_secret`.             |
| `PUT /api/oauth/apps/{id}`                  | Edit name, description, and redirect URIs.                         |
| `DELETE /api/oauth/apps/{id}`               | Delete an application and everything under it.                     |
| `POST /api/oauth/apps/{id}/secret`          | Replace the client secret.                                         |
| `DELETE /api/admin/oauth/apps/{id}`         | Take any application down. Super admins and the service key only.  |

Field errors on create and edit come back together in one `422 VALIDATION_FAILED` with `details: [{ field, message }]`. The 11th application answers `409 CONFLICT`. Editing, deleting, or rotating an application you do not own answers 404, not 403.

### Layout

Paths are relative to `backend/crates/`.

| Path                                 | Holds                                                             |
|--------------------------------------|-------------------------------------------------------------------|
| `app/src/services/oauth/protocol.rs` | Rules with no database: lifetimes and limits, token prefixes, scopes, redirect-URI validation, PKCE. |
| `app/src/services/oauth/tokens.rs`   | Client authentication, the code exchange, refresh, revoke, `userinfo`. |
| `app/src/services/oauth/error.rs`    | The protocol error type with its RFC codes and descriptions.      |
| `app/src/services/oauth/dto.rs`      | Request and response shapes.                                      |
| `app/src/services/oauth/consent.rs`  | Validating an authorization request, approving and declining it.  |
| `app/src/services/oauth/apps.rs`     | Registering, editing, and deleting applications.                  |
| `db/src/repos/oauth.rs`              | Every SQL statement.                                              |
| `api/src/routes/oauth/`              | Routes: `protocol.rs`, `consent.rs`, `apps.rs`.                   |
| `api/src/oauth_error.rs`             | The RFC error body, status codes, and `WWW-Authenticate` headers. |

The four tables are created by the baseline migration, `0001_baseline.sql`. The consent page is the web client's `/oauth/consent` route (`frontend/src/components/oauth`).

### Database

| Table          | Purpose                                                                                  |
|----------------|------------------------------------------------------------------------------------------|
| `oauth_apps`   | One row per application: owner, name, description, redirect URIs, secret hash.           |
| `oauth_grants` | One row per (application, player): the access a player gave. Unique on the pair.         |
| `oauth_codes`  | Authorization codes by hash, with the redirect URI, scope, and PKCE challenge they were issued for. |
| `oauth_tokens` | One row per session: access and refresh hashes with their expiries.                      |

Every foreign key cascades on delete: users to applications and grants, applications to grants, grants to codes and tokens. So deleting an account removes the applications it registered, the access it granted, and every token under either.

### Behaviours worth knowing

- **Secrets at rest.** Client secrets, codes, access tokens, and refresh tokens are 32 random bytes each, and only their SHA-256 digests are stored.
- **A refresh updates the session row in place.** There is no refresh-token family and no reuse detection: an old refresh token simply no longer matches. Replay detection exists for authorization codes only, and only until the spent code's row is swept.
- **Housekeeping runs on approval.** Expired codes and sessions whose refresh token has expired are deleted whenever a player approves a request. There is no scheduled job.
- **`last_used_at` on a grant** is stamped when tokens are issued or refreshed, not on `userinfo` calls.
- **`userinfo` does not check the issuing application.** Any live access token with `identify` is answered.
- **Why `/oauth/authorize` redirects to `/oauth/consent`.** The web client's router rewrites query values through `JSON.parse` (`state=1e3` becomes `1000`). `/oauth/authorize` is a server route that packs the raw query string into one opaque value so `state` returns to the application byte for byte. See `frontend/src/lib/oauth-request.ts`.
- **The consent page cannot be framed.** It sends `X-Frame-Options: DENY` and `frame-ancestors 'none'`.
- **Taking an application down.** `DELETE /api/admin/oauth/apps/{id}`, where `id` is the application's `client_id`, signs every player out of it. The `client_id` is in the authorization link the application sends people to.
- **`server.public_api_url`** in `config/myrtle.toml` sets the base of `avatar_url`. Empty, it is derived from the request's `Host` header (`https` when `X-Forwarded-Proto` says so), and falls back to `https://api.myrtle.moe` when there is no `Host`.
- **Server-prefixed paths route too.** `/api/en/oauth/token` is rewritten to `/api/oauth/token`, like every other API path.

### Tests

```bash
cargo test -p myrtle-app --test oauth_protocol   # Pure rules, no database
cargo test -p myrtle-app --test oauth_apps       # Field validation and limits
cargo test -p myrtle-db  --test oauth            # Repository statements
cargo test -p myrtle-api --test oauth_flow       # End to end through the router
```

All four need `DATABASE_URL` pointing at a migrated schema to compile (see the [backend README](README.md#1-load-the-schema)). At run time only `oauth_protocol` does without Postgres; the other three create fresh databases on that server. `a_player_signs_in_to_a_third_party_application` in `api/tests/oauth_flow.rs` is the whole flow in one test and the best worked example of every request and response on this page.

## License

License TBD - see the [project root](../README.md).

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
