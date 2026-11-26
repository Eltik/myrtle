import { createServerFn } from "@tanstack/react-start";
import { z } from "zod";
import { APIError, parseError } from "#/lib/api/_shared";
import { requireSiteToken } from "#/lib/api/_shared.server";
import { backendFetch } from "#/lib/fetch";
import type { AuthorizeCheck } from "#/types/generated/AuthorizeCheck";
import type { AuthorizeDecision } from "#/types/generated/AuthorizeDecision";
import type { OAuthApp } from "#/types/generated/OAuthApp";
import type { OAuthAppWithSecret } from "#/types/generated/OAuthAppWithSecret";
import type { OAuthAuthorization } from "#/types/generated/OAuthAuthorization";

/**
 * "Sign in with myrtle.moe": the site's own half of the OAuth provider.
 *
 * Every function here runs on the SSR host with the visitor's `site_token`
 * cookie. The endpoints a third party calls (`/oauth/token`, `/oauth/revoke`,
 * `/oauth/userinfo`) are not here: its server reaches the backend directly.
 */

// One parameter of an authorization request. The backend owns the rules; the
// bound only keeps a hostile link from posting megabytes through this host.
const param = z.string().max(2048).optional();

const authorizeParamsSchema = z.object({
    response_type: param,
    client_id: param,
    redirect_uri: param,
    scope: param,
    state: param,
    code_challenge: param,
    code_challenge_method: param,
});

/** An authorization request as the other site sent it; `AuthorizeParams` on the backend. */
export type AuthorizeRequest = z.infer<typeof authorizeParamsSchema>;

const appInputSchema = z.object({
    name: z.string().max(256),
    description: z.string().max(2048),
    redirect_uris: z.array(z.string().max(2048)).max(64),
});

export type OAuthAppFields = z.infer<typeof appInputSchema>;

const idSchema = z.object({ id: z.uuid() });

/**
 * A request to the backend as the signed-in visitor. Throws `APIError(401)`
 * when signed out. A 422 carries one entry per rejected field; its message is
 * rebuilt from those, because the envelope's own says only "validation
 * failed" and the form needs to say which redirect URI and why.
 */
async function asUser(path: string, init: { method?: string; body?: unknown } = {}): Promise<Response> {
    const res = await backendFetch(path, {
        method: init.method ?? "GET",
        bearerToken: requireSiteToken(),
        body: init.body === undefined ? undefined : JSON.stringify(init.body),
    });
    if (res.ok) return res;
    if (res.status === 422) {
        const body = (await res
            .clone()
            .json()
            .catch(() => null)) as { error?: { details?: { message?: string }[] } } | null;
        const reasons = (body?.error?.details ?? []).map((d) => d.message).filter((m): m is string => Boolean(m));
        if (reasons.length > 0) throw new APIError(422, reasons.join("\n"), "VALIDATION_FAILED");
    }
    throw await parseError(res);
}

/**
 * What the consent page should do with an authorization request: ask the
 * visitor, send the browser back with an error, or (`invalid`) tell the
 * visitor the request cannot be honoured at all.
 *
 * `invalid` is the backend's 400: an unknown application, or a redirect URI
 * it never registered, so there is nowhere trustworthy to send the browser.
 * It is a RESULT, not a thrown error, on purpose. A thrown error reaches the
 * browser with its message only (its status does not survive the server
 * function boundary), and the page must tell "this link is bad" from "the
 * server is having a moment, try again".
 */
export type AuthorizeCheckResult = AuthorizeCheck | { status: "invalid"; reason: string };

export const checkAuthorizationFn = createServerFn({ method: "POST" })
    .inputValidator((d: AuthorizeRequest) => authorizeParamsSchema.parse(d))
    .handler(async ({ data }): Promise<AuthorizeCheckResult> => {
        const query = new URLSearchParams();
        for (const [key, value] of Object.entries(data)) {
            if (value !== undefined) query.set(key, value);
        }
        try {
            const res = await asUser(`/oauth/authorize?${query.toString()}`);
            return (await res.json()) as AuthorizeCheck;
        } catch (err) {
            if (err instanceof APIError && err.status === 400) return { status: "invalid", reason: err.message };
            throw err;
        }
    });

/** Record the visitor's answer; the result is where their browser goes next. */
export const decideAuthorizationFn = createServerFn({ method: "POST" })
    .inputValidator((d: AuthorizeRequest & { approve: boolean }) => authorizeParamsSchema.extend({ approve: z.boolean() }).parse(d))
    .handler(async ({ data }): Promise<AuthorizeDecision> => {
        const res = await asUser("/oauth/authorize", { method: "POST", body: data });
        return (await res.json()) as AuthorizeDecision;
    });

/** The applications the visitor has authorized, most recently used first. */
export const listAuthorizationsFn = createServerFn({ method: "GET" }).handler(async (): Promise<OAuthAuthorization[]> => {
    const res = await asUser("/oauth/authorizations");
    return (await res.json()) as OAuthAuthorization[];
});

/** Withdraw the visitor's authorization of one application; its tokens stop working at once. */
export const revokeAuthorizationFn = createServerFn({ method: "POST" })
    .inputValidator((d: { id: string }) => idSchema.parse(d))
    .handler(async ({ data }): Promise<void> => {
        await asUser(`/oauth/authorizations/${data.id}`, { method: "DELETE" });
    });

/** The applications the visitor registered. Never carries a secret. */
export const listOAuthAppsFn = createServerFn({ method: "GET" }).handler(async (): Promise<OAuthApp[]> => {
    const res = await asUser("/oauth/apps");
    return (await res.json()) as OAuthApp[];
});

/** Register an application. The response is the one time its client secret can be read. */
export const createOAuthAppFn = createServerFn({ method: "POST" })
    .inputValidator((d: OAuthAppFields) => appInputSchema.parse(d))
    .handler(async ({ data }): Promise<OAuthAppWithSecret> => {
        const res = await asUser("/oauth/apps", { method: "POST", body: data });
        return (await res.json()) as OAuthAppWithSecret;
    });

export const updateOAuthAppFn = createServerFn({ method: "POST" })
    .inputValidator((d: OAuthAppFields & { id: string }) => appInputSchema.extend(idSchema.shape).parse(d))
    .handler(async ({ data }): Promise<OAuthApp> => {
        const { id, ...fields } = data;
        const res = await asUser(`/oauth/apps/${id}`, { method: "PUT", body: fields });
        return (await res.json()) as OAuthApp;
    });

/** Replace an application's client secret. The old one stops working at once. */
export const rotateOAuthSecretFn = createServerFn({ method: "POST" })
    .inputValidator((d: { id: string }) => idSchema.parse(d))
    .handler(async ({ data }): Promise<OAuthAppWithSecret> => {
        const res = await asUser(`/oauth/apps/${data.id}/secret`, { method: "POST" });
        return (await res.json()) as OAuthAppWithSecret;
    });

/** Delete an application, signing every player out of it. */
export const deleteOAuthAppFn = createServerFn({ method: "POST" })
    .inputValidator((d: { id: string }) => idSchema.parse(d))
    .handler(async ({ data }): Promise<void> => {
        await asUser(`/oauth/apps/${data.id}`, { method: "DELETE" });
    });
