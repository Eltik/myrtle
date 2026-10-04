import { createServerFn } from "@tanstack/react-start";
import { parseError } from "#/lib/api/_shared";
import { requireSiteToken } from "#/lib/api/_shared.server";
import { backendFetch } from "#/lib/fetch";
import type { DisconnectResult } from "#/types/generated/DisconnectResult";
import type { StatusOk } from "#/types/generated/StatusOk";

export interface IUpdateUserSettingsInput {
    public_profile: boolean;
    store_gacha: boolean;
    share_stats: boolean;
}

/**
 * POST to the backend as the signed-in user. Throws `APIError(401)` when
 * signed out, and the backend's own `APIError` (code and message unwrapped
 * from its error body) on any non-2xx answer.
 */
async function postAsUser(path: string, body?: unknown): Promise<Response> {
    const res = await backendFetch(path, {
        method: "POST",
        bearerToken: requireSiteToken(),
        body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!res.ok) throw await parseError(res);
    return res;
}

export const updateUserSettingsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpdateUserSettingsInput) => data)
    .handler(async ({ data }) => {
        const res = await postAsUser("/auth/update-settings", data);
        return (await res.json()) as StatusOk;
    });

/**
 * Forget the stored Yostar credentials for the signed-in account.
 *
 * Leaves the site session and the already-synced data alone - it only revokes
 * our ability to reach Yostar. `removed` reports whether anything was stored.
 */
export const disconnectGameAccountFn = createServerFn({ method: "POST" }).handler(async (): Promise<{ removed: boolean }> => {
    const res = await postAsUser("/auth/disconnect");
    // `removed` is required on the generated type; the backend always sends it.
    const body = (await res.json()) as DisconnectResult;
    return { removed: body.removed };
});

export const refreshRosterFn = createServerFn({ method: "POST" }).handler(async (): Promise<{ status: string }> => {
    await postAsUser("/refresh");
    return { status: "ok" };
});
