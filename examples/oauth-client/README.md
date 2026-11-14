# OAuth demo: "Sign in with myrtle.moe"

A complete OAuth 2.0 client for myrtle.moe in one file, [`server.mjs`](server.mjs).
No dependencies; Node 18 or newer, or Bun.

It does every step a real site needs:

1. sends the player to the myrtle.moe consent page, with a `state` and PKCE;
2. handles the redirect back, including "the player pressed Cancel";
3. exchanges the code for tokens, server to server;
4. reads the player's identity from `/api/oauth/userinfo`;
5. refreshes the access token when it runs out;
6. revokes the tokens on sign-out.

The protocol itself is documented in [`backend/OAUTH.md`](../../backend/OAUTH.md).

## Run it

1. On myrtle.moe, open **Settings**, then **Applications**, and create an
   application with the redirect URI `http://localhost:4000/callback`.
2. Copy the client ID and the client secret it shows you.
3. Start the demo:

   ```bash
   MYRTLE_CLIENT_ID=... MYRTLE_CLIENT_SECRET=... node server.mjs
   ```

4. Open <http://localhost:4000> and press "Sign in with myrtle.moe".

## Configuration

| Variable | Default | |
|---|---|---|
| `MYRTLE_CLIENT_ID` | required | The application's client ID. |
| `MYRTLE_CLIENT_SECRET` | required | Its client secret. |
| `PORT` | `4000` | Port to listen on. |
| `APP_URL` | `http://localhost:$PORT` | Where the player's browser reaches this app. The redirect URI is `$APP_URL/callback` and must be registered on the application exactly. |
| `MYRTLE_SITE_URL` | `https://myrtle.moe` | The website, which hosts the consent page. |
| `MYRTLE_API_URL` | `https://api.myrtle.moe` | The API, which hosts the token, revoke and userinfo endpoints. |

Against a local myrtle stack (frontend on 3000, backend on 3060):

```bash
MYRTLE_SITE_URL=http://localhost:3000 MYRTLE_API_URL=http://localhost:3060 \
MYRTLE_CLIENT_ID=... MYRTLE_CLIENT_SECRET=... node server.mjs
```

## What to change for a real site

- Sessions and pending sign-ins live in memory here, so a restart signs
  everyone out. Keep them in your database, and encrypt stored tokens.
- Key your users on `id` from the userinfo response. `username` is the
  in-game nickname: it is not unique and it can change.
- If you only need to know who signed in, you can call `userinfo` once and
  discard the tokens instead of storing them.
