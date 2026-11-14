# Sign in with myrtle.moe

myrtle.moe is an OAuth 2.0 provider. Another site can send a player to
myrtle.moe, the player approves, and the site learns who they are: a stable
account id, their in-game nickname, their game UID and server, and their
avatar. Nothing else is shared and nothing can be changed.

It is the standard authorization-code grant
([RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749)) with PKCE
([RFC 7636](https://datatracker.ietf.org/doc/html/rfc7636)), so any OAuth 2.0
client library works. A complete client in one dependency-free file is in
[`examples/oauth-client`](../examples/oauth-client).

## Endpoints

| | |
|---|---|
| Authorization | `GET https://myrtle.moe/oauth/authorize` |
| Token | `POST https://api.myrtle.moe/api/oauth/token` |
| Revocation | `POST https://api.myrtle.moe/api/oauth/revoke` |
| User info | `GET https://api.myrtle.moe/api/oauth/userinfo` |
| Scope | `identify` |

The authorization endpoint is on the website; the other three are on the API
host. Their request and response schemas are also in the API reference at
`https://api.myrtle.moe/docs`, under the `oauth` tag.

## 1. Register an application

Sign in to myrtle.moe, open **Settings**, then **Applications**, and create an
application under "Your applications". You give it:

- a **name** and an optional **description**, shown to players on the consent
  screen together with your in-game name;
- one or more **redirect URIs**: where a player is sent back to. Each must be
  an absolute `https` URL without a fragment. `http` is accepted for
  `localhost`, `127.0.0.1` and `[::1]`, for development.

You get a **client ID** and a **client secret**. The secret is shown once and
stored only as a hash. If you lose it, "New secret" replaces it; the old one
stops working at once, and people who already signed in stay signed in. Keep
the secret on your server. It does not belong in a browser or a mobile app.

An account can register up to 10 applications, each with up to 10 redirect
URIs.

## 2. Send the player to myrtle.moe

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

| Parameter | |
|---|---|
| `response_type` | Required. `code`. |
| `client_id` | Required. |
| `redirect_uri` | One of your registered URIs, character for character. Optional only if you registered exactly one. |
| `scope` | Optional. `identify` is the only scope and the default. |
| `state` | Recommended. A random value you tie to the player's browser session and check when they come back. Up to 1024 bytes, returned unchanged. |
| `code_challenge` | Recommended (PKCE). The unpadded base64url SHA-256 of a random `code_verifier` (43 to 128 characters) that you keep. |
| `code_challenge_method` | `S256`, when you send a challenge. `plain` is not accepted. |

The player signs in if they are not already, sees your application's name and
what it will learn, and chooses Authorize or Cancel. They are asked every time.

If `client_id` is unknown or `redirect_uri` is not registered, the player is
shown an error on myrtle.moe and is not redirected anywhere. Every other
problem is reported to your redirect URI, as below.

## 3. Handle the redirect back

The player returns to your `redirect_uri` with either a code:

```text
https://your.site/auth/myrtle/callback?code=myrtle_ac_...&state=RANDOM_VALUE_YOU_STORED
```

or an error (RFC 6749 section 4.1.2.1):

```text
https://your.site/auth/myrtle/callback?error=access_denied&error_description=...&state=...
```

| `error` | Meaning |
|---|---|
| `access_denied` | The player pressed Cancel. |
| `invalid_request` | A parameter is missing or malformed. `error_description` says which. |
| `unsupported_response_type` | `response_type` was not `code`. |
| `invalid_scope` | An unknown scope was requested. |

Check that `state` is the value you stored for this browser before doing
anything else. If it is not, stop: someone may be trying to complete a sign-in
they started elsewhere.

## 4. Exchange the code for tokens

From your server, within five minutes:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/token \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d grant_type=authorization_code \
  -d code="$CODE" \
  -d redirect_uri=https://your.site/auth/myrtle/callback \
  -d code_verifier="$CODE_VERIFIER"
```

The body is `application/x-www-form-urlencoded`. Authenticate with HTTP Basic
(`client_id` as the username, `client_secret` as the password), or send
`client_id` and `client_secret` as two more form fields.

```json
{
  "access_token": "myrtle_at_...",
  "token_type": "Bearer",
  "expires_in": 604800,
  "refresh_token": "myrtle_rt_...",
  "scope": "identify"
}
```

- A code works **once**. Presenting a code a second time fails, and also
  revokes the tokens the first exchange issued, because a code seen twice may
  have leaked.
- `code_verifier` is required if the authorization request carried a
  `code_challenge`.
- `redirect_uri`, when sent, must be the one the code was issued for.

## 5. Read the player's identity

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

| Field | |
|---|---|
| `id` | The myrtle.moe account id. Stable for the life of the account. **Key your connected account on this.** |
| `username` | The player's in-game nickname. Not unique, and it changes when they rename in game. For an account whose nickname has not synced yet it is the game UID. |
| `tag` | The digits after the `#` in the in-game name, or `null`. |
| `uid` | The in-game UID on `server`. `https://myrtle.moe/user/{uid}` is the player's profile page; it opens only if they made their profile public. |
| `server` | `en`, `jp`, `kr`, `cn`, `bili` or `tw`. |
| `avatar_id` | The avatar the player picked in game, or `null` if they never set one. |
| `avatar_url` | A square PNG of that avatar (a default one when `avatar_id` is `null`). Safe to hotlink. |

## Token lifetimes and refreshing

| Token | Lifetime |
|---|---|
| Authorization code | 5 minutes, single use |
| Access token | 7 days |
| Refresh token | 90 days from its last use, single use |

If all you need is to know who signed in, call `userinfo` once and discard the
tokens. To keep a connection alive (for example to refresh a displayed
nickname), store the refresh token and trade it for a new pair when the access
token expires or `userinfo` answers 401:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/token \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d grant_type=refresh_token \
  -d refresh_token="$REFRESH_TOKEN"
```

The response has the same shape as before. **Every refresh returns a new
refresh token and retires the one you sent**, so store the new one. The old
access token stops working too.

## Signing out

To end a sign-in from your side, revoke either of its tokens
([RFC 7009](https://datatracker.ietf.org/doc/html/rfc7009)). Both stop working:

```bash
curl -X POST https://api.myrtle.moe/api/oauth/revoke \
  -u "$CLIENT_ID:$CLIENT_SECRET" \
  -d token="$REFRESH_TOKEN"
```

The answer is `200` with an empty body, also for a token that was already dead.

A player can remove your application's access themselves, in Settings under
Applications. Your tokens for them then answer `invalid_token` /
`invalid_grant`; treat that as signed out.

## Errors

The token, revocation and userinfo endpoints answer errors in the OAuth format
(RFC 6749 section 5.2), not in the envelope the rest of the API uses:

```json
{ "error": "invalid_grant", "error_description": "the authorization code is invalid, expired or already used" }
```

| Status | `error` | Meaning |
|---|---|---|
| 400 | `invalid_request` | A parameter is missing, or the body is not form-encoded. |
| 400 | `invalid_grant` | The code or refresh token is invalid, expired, already used, or belongs to another application; or the `redirect_uri` or `code_verifier` does not match. |
| 400 | `unsupported_grant_type` | `grant_type` is neither `authorization_code` nor `refresh_token`. |
| 400 | `invalid_scope` | An unknown scope. |
| 401 | `invalid_client` | Unknown `client_id` or wrong `client_secret`. |
| 401 | `invalid_token` | (`userinfo`) The access token is missing, expired or revoked. |
| 403 | `insufficient_scope` | (`userinfo`) The token was not granted `identify`. |
| 429 | | Rate limited. This one uses the API's usual envelope and a `Retry-After` header. The limit is per IP address. |
| 5xx | `server_error`, `temporarily_unavailable` | Retry. |

## What an access token can and cannot do

An access token is accepted by `GET /api/oauth/userinfo` and nowhere else. It
is not the session token the website uses: it cannot read a private profile,
change settings, trigger a roster sync or delete an account. Likewise, a
website session token is not accepted by `userinfo`.

---

## For maintainers

- **Layout.** Rules with no database in `src/core/auth/oauth.rs` (secrets,
  scopes, redirect-URI validation, PKCE). Statements in
  `src/database/queries/oauth.rs`. Orchestration in
  `src/app/services/oauth.rs`. Routes in `src/app/routes/oauth/`. Schema in
  migration `v046_oauth.sql`. The consent page is the web client's
  `/oauth/consent` route (`frontend/src/components/oauth`).
- **Secrets at rest.** Client secrets, codes, access tokens and refresh tokens
  are 32 random bytes each and are stored only as SHA-256 hashes.
- **Why `/oauth/authorize` redirects to `/oauth/consent`.** The web client's
  router rewrites query values through `JSON.parse` (`state=1e3` becomes
  `1000`). `/oauth/authorize` is a server route that packs the raw query
  string into one opaque value so `state` returns to the application byte for
  byte. See `frontend/src/lib/oauth-request.ts`.
- **Housekeeping.** Expired codes and sessions are deleted whenever a player
  approves a request; there is no scheduled job.
- **Taking an application down.** A super admin can delete any application
  with `DELETE /api/admin/oauth/apps/{client_id}`. Every player is signed out
  of it. The `client_id` is in the authorization link the application sends
  people to.
- **Account deletion** removes the applications the account registered, the
  access it granted, and every token under either (foreign-key cascades).
- **`PUBLIC_API_URL`** sets the base of `avatar_url`. Unset, it is derived
  from the request's `Host` header.
- **Tests.** `cargo test --lib oauth` (no database) and
  `cargo test --test oauth_flow_test -- --include-ignored` (local Postgres).
