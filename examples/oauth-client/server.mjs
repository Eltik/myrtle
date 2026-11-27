// "Sign in with myrtle.moe": a complete OAuth 2.0 client in one file.
//
// No dependencies, Node 18 or newer (or Bun). It shows every step a site
// needs: sending the player to the consent page with PKCE and a `state`,
// handling the redirect back, exchanging the code, reading the identity,
// refreshing the token and revoking it on sign-out.
//
//   MYRTLE_CLIENT_ID=... MYRTLE_CLIENT_SECRET=... node server.mjs
//
// See README.md next to this file, and backend/OAUTH.md for the protocol.

import { createHash, randomBytes, timingSafeEqual } from "node:crypto";
import { createServer } from "node:http";

const PORT = Number(process.env.PORT ?? 4000);
// Where this app is reachable from the player's browser.
const APP_URL = (process.env.APP_URL ?? `http://localhost:${PORT}`).replace(/\/+$/, "");
// The website (consent page) and the API (token, userinfo) are separate hosts.
const SITE_URL = (process.env.MYRTLE_SITE_URL ?? "https://myrtle.moe").replace(/\/+$/, "");
const API_URL = (process.env.MYRTLE_API_URL ?? "https://api.myrtle.moe").replace(/\/+$/, "");
const CLIENT_ID = process.env.MYRTLE_CLIENT_ID;
const CLIENT_SECRET = process.env.MYRTLE_CLIENT_SECRET;

// Must be registered on the application, character for character.
const REDIRECT_URI = `${APP_URL}/callback`;

if (!CLIENT_ID || !CLIENT_SECRET) {
    console.error("Set MYRTLE_CLIENT_ID and MYRTLE_CLIENT_SECRET (register an application in myrtle.moe Settings, under Applications).");
    process.exit(1);
}

// In memory, which is fine for a demo: a restart signs everyone out. A real
// site keeps sessions in its database and the tokens encrypted at rest.
/** @type {Map<string, { user: any, accessToken: string, refreshToken: string, expiresAt: number }>} */
const sessions = new Map();
/** Sign-ins that left for the consent page and have not come back. @type {Map<string, { verifier: string, createdAt: number }>} */
const pending = new Map();
const PENDING_TTL_MS = 10 * 60 * 1000;

const base64url = (bytes) => Buffer.from(bytes).toString("base64url");
const randomToken = () => base64url(randomBytes(32));

/** HTTP Basic credentials for the token and revoke endpoints. */
const clientAuth = `Basic ${Buffer.from(`${encodeURIComponent(CLIENT_ID)}:${encodeURIComponent(CLIENT_SECRET)}`).toString("base64")}`;

/**
 * POST a form to one of the protocol endpoints as this application.
 * Errors come back as `{ error, error_description }` (RFC 6749 5.2).
 */
async function oauthPost(path, fields) {
    const res = await fetch(`${API_URL}/api/oauth/${path}`, {
        method: "POST",
        headers: { Authorization: clientAuth, "Content-Type": "application/x-www-form-urlencoded" },
        body: new URLSearchParams(fields),
    });
    const text = await res.text();
    const body = text ? JSON.parse(text) : {};
    if (!res.ok) throw new Error(`${body.error ?? res.status}: ${body.error_description ?? "request failed"}`);
    return body;
}

/** `GET /api/oauth/userinfo`. `null` when the access token is no longer good. */
async function fetchUser(accessToken) {
    const res = await fetch(`${API_URL}/api/oauth/userinfo`, { headers: { Authorization: `Bearer ${accessToken}` } });
    if (res.status === 401) return null;
    if (!res.ok) throw new Error(`userinfo failed: ${res.status}`);
    return res.json();
}

function storeTokens(session, tokens) {
    session.accessToken = tokens.access_token;
    // Refresh tokens are single use: every refresh returns the next one.
    session.refreshToken = tokens.refresh_token;
    session.expiresAt = Date.now() + tokens.expires_in * 1000;
}

/** The player's current identity, refreshing the access token first if it has run out or been rejected. */
async function currentUser(session) {
    let user = Date.now() < session.expiresAt ? await fetchUser(session.accessToken) : null;
    if (!user) {
        storeTokens(session, await oauthPost("token", { grant_type: "refresh_token", refresh_token: session.refreshToken }));
        user = await fetchUser(session.accessToken);
    }
    if (!user) throw new Error("the access token was rejected right after a refresh");
    session.user = user;
    return user;
}

// ---------------------------------------------------------------------------
// Plumbing: cookies, HTML
// ---------------------------------------------------------------------------

function cookies(req) {
    const out = {};
    for (const part of (req.headers.cookie ?? "").split(";")) {
        const [name, ...rest] = part.trim().split("=");
        if (name) out[name] = decodeURIComponent(rest.join("="));
    }
    return out;
}

const secure = APP_URL.startsWith("https://") ? "; Secure" : "";
const setCookie = (name, value, maxAgeSeconds) => `${name}=${encodeURIComponent(value)}; Path=/; HttpOnly; SameSite=Lax; Max-Age=${maxAgeSeconds}${secure}`;

const escapeHtml = (value) => String(value ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

function page(title, body) {
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escapeHtml(title)}</title>
<style>
  :root { color-scheme: light dark; --bg: #fafafa; --card: #fff; --line: #dcdce0; --text: #18181b; --muted: #63636e; --accent: #d1343a; }
  @media (prefers-color-scheme: dark) { :root { --bg: #111113; --card: #1b1b1e; --line: #33333a; --text: #f4f4f5; --muted: #a1a1aa; --accent: #f08a86; } }
  * { box-sizing: border-box; }
  body { margin: 0; min-height: 100vh; display: grid; place-items: center; padding: 24px 16px; background: var(--bg); color: var(--text); font: 15px/1.55 system-ui, sans-serif; }
  main { width: 100%; max-width: 440px; background: var(--card); border: 1px solid var(--line); border-radius: 14px; padding: 24px; }
  h1 { margin: 0 0 6px; font-size: 20px; }
  p { margin: 0 0 16px; color: var(--muted); }
  .who { display: flex; align-items: center; gap: 14px; margin: 18px 0; }
  .who img { width: 64px; height: 64px; border-radius: 50%; border: 1px solid var(--line); background: var(--bg); }
  .who strong { font-size: 17px; }
  .who span { color: var(--muted); }
  dl { margin: 0 0 18px; display: grid; grid-template-columns: auto 1fr; gap: 6px 14px; font-size: 13.5px; }
  dt { color: var(--muted); }
  dd { margin: 0; font-family: ui-monospace, monospace; overflow-wrap: anywhere; }
  .row { display: flex; flex-wrap: wrap; gap: 8px; }
  form { margin: 0; }
  a.button, button { display: inline-flex; align-items: center; justify-content: center; min-height: 40px; padding: 0 16px; border-radius: 9px; border: 1px solid var(--line); background: transparent; color: var(--text); font: inherit; font-weight: 500; text-decoration: none; cursor: pointer; }
  a.button.primary, button.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
  @media (prefers-color-scheme: dark) { a.button.primary, button.primary { color: #111; } }
  .error { border: 1px solid var(--accent); border-radius: 9px; padding: 10px 12px; color: var(--text); margin-bottom: 16px; overflow-wrap: anywhere; }
</style>
</head>
<body><main>${body}</main></body>
</html>`;
}

function send(res, status, html, headers = {}) {
    res.writeHead(status, { "Content-Type": "text/html; charset=utf-8", "Cache-Control": "no-store", ...headers });
    res.end(html);
}

const redirect = (res, location, headers = {}) => {
    res.writeHead(302, { Location: location, "Cache-Control": "no-store", ...headers });
    res.end();
};

function signedOutPage(error) {
    return page(
        "OAuth demo",
        `${error ? `<div class="error">${escapeHtml(error)}</div>` : ""}
        <h1>OAuth demo</h1>
        <p>A tiny site that uses myrtle.moe to sign people in. It learns your nickname, UID, server and avatar, and nothing else.</p>
        <a class="button primary" href="/login">Sign in with myrtle.moe</a>`,
    );
}

function signedInPage(user) {
    const name = `${escapeHtml(user.username)}${user.tag ? `<span>#${escapeHtml(user.tag)}</span>` : ""}`;
    return page(
        `Signed in as ${user.username}`,
        `<h1>You're signed in</h1>
        <div class="who">
            <img src="${escapeHtml(user.avatar_url)}" alt="" width="64" height="64">
            <div><strong>${name}</strong><br><span>${escapeHtml(user.server.toUpperCase())} · UID ${escapeHtml(user.uid)}</span></div>
        </div>
        <dl>
            <dt>id</dt><dd>${escapeHtml(user.id)}</dd>
            <dt>username</dt><dd>${escapeHtml(user.username)}</dd>
            <dt>tag</dt><dd>${escapeHtml(user.tag ?? "null")}</dd>
            <dt>uid</dt><dd>${escapeHtml(user.uid)}</dd>
            <dt>server</dt><dd>${escapeHtml(user.server)}</dd>
            <dt>avatar_id</dt><dd>${escapeHtml(user.avatar_id ?? "null")}</dd>
        </dl>
        <div class="row">
            <form method="post" action="/refresh"><button>Refresh from myrtle.moe</button></form>
            <form method="post" action="/logout"><button>Sign out</button></form>
        </div>`,
    );
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/** Step 1: send the player to the consent page. */
function login(res) {
    const now = Date.now();
    for (const [key, entry] of pending) if (now - entry.createdAt > PENDING_TTL_MS) pending.delete(key);

    // `state` ties the redirect back to this browser, so nobody can finish a
    // sign-in they started somewhere else in this one. PKCE ties the code to
    // this server: without `verifier` a stolen code is worthless.
    const state = randomToken();
    const verifier = randomToken();
    pending.set(state, { verifier, createdAt: now });

    const authorize = new URL(`${SITE_URL}/oauth/authorize`);
    authorize.search = new URLSearchParams({
        response_type: "code",
        client_id: CLIENT_ID,
        redirect_uri: REDIRECT_URI,
        scope: "identify",
        state,
        code_challenge: base64url(createHash("sha256").update(verifier).digest()),
        code_challenge_method: "S256",
    }).toString();
    redirect(res, authorize.toString(), { "Set-Cookie": setCookie("oauth_state", state, PENDING_TTL_MS / 1000) });
}

/** Step 2: the player is back, with `code` (approved) or `error` (declined, or a bad request). */
async function callback(req, res, query) {
    const clearState = setCookie("oauth_state", "", 0);
    const fail = (message) => send(res, 400, signedOutPage(message), { "Set-Cookie": clearState });

    const state = query.get("state") ?? "";
    const expected = cookies(req).oauth_state ?? "";
    const entry = pending.get(state);
    pending.delete(state);
    // The state has to be the one THIS browser was given.
    const sameState = state.length > 0 && state.length === expected.length && timingSafeEqual(Buffer.from(state), Buffer.from(expected));
    if (!sameState || !entry) return fail("That sign-in link is stale or was not started here. Try again.");

    if (query.has("error")) {
        const declined = query.get("error") === "access_denied";
        return fail(declined ? "You cancelled the sign-in." : `myrtle.moe refused the request: ${query.get("error_description") ?? query.get("error")}`);
    }
    const code = query.get("code");
    if (!code) return fail("The redirect carried no code.");

    try {
        // Step 3: trade the code for tokens. Server to server, with the
        // client secret and the PKCE verifier; the code works exactly once.
        const tokens = await oauthPost("token", { grant_type: "authorization_code", code, redirect_uri: REDIRECT_URI, code_verifier: entry.verifier });

        // Step 4: ask who it is.
        const user = await fetchUser(tokens.access_token);
        if (!user) throw new Error("the new access token was rejected");

        const session = { user };
        storeTokens(session, tokens);
        const sid = randomToken();
        sessions.set(sid, session);
        redirect(res, "/", { "Set-Cookie": [setCookie("sid", sid, 30 * 24 * 60 * 60), clearState] });
    } catch (err) {
        fail(`Sign-in failed. ${err.message}`);
    }
}

async function handle(req, res) {
    const url = new URL(req.url ?? "/", APP_URL);
    const sid = cookies(req).sid;
    const session = sid ? sessions.get(sid) : undefined;

    if (req.method === "GET" && url.pathname === "/") {
        return send(res, 200, session ? signedInPage(session.user) : signedOutPage());
    }
    if (req.method === "GET" && url.pathname === "/login") return login(res);
    if (req.method === "GET" && url.pathname === "/callback") return callback(req, res, url.searchParams);

    if (req.method === "POST" && url.pathname === "/refresh") {
        if (!session) return redirect(res, "/");
        try {
            await currentUser(session);
            return redirect(res, "/");
        } catch (err) {
            // The player removed this app in their myrtle.moe settings, or the
            // refresh token ran out: the session here is over.
            sessions.delete(sid);
            return send(res, 401, signedOutPage(`Your sign-in is no longer valid. ${err.message}`), { "Set-Cookie": setCookie("sid", "", 0) });
        }
    }

    if (req.method === "POST" && url.pathname === "/logout") {
        if (session) {
            sessions.delete(sid);
            // Tell myrtle.moe the tokens are done with (RFC 7009). Best effort:
            // the local session is gone either way.
            await oauthPost("revoke", { token: session.refreshToken }).catch((err) => console.warn("revoke failed:", err.message));
        }
        return redirect(res, "/", { "Set-Cookie": setCookie("sid", "", 0) });
    }

    send(res, 404, page("Not found", `<h1>Not found</h1><p><a href="/">Back to the demo</a></p>`));
}

createServer((req, res) => {
    handle(req, res).catch((err) => {
        console.error(err);
        if (!res.headersSent) send(res, 500, page("Error", `<h1>Something went wrong</h1><p>${escapeHtml(err.message)}</p>`));
    });
}).listen(PORT, () => {
    console.log(`OAuth demo on ${APP_URL}`);
    console.log(`  redirect URI to register: ${REDIRECT_URI}`);
    console.log(`  consent page: ${SITE_URL}/oauth/authorize`);
    console.log(`  API: ${API_URL}/api/oauth/{token,revoke,userinfo}`);
});
