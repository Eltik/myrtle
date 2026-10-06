import { createServerFn } from "@tanstack/react-start";
import { parseError } from "#/lib/api/_shared";
import { requireSiteToken } from "#/lib/api/_shared.server";
import { backendFetch } from "#/lib/fetch";
import type { DisconnectResult } from "#/types/generated/DisconnectResult";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { ProfileShowcase } from "#/types/generated/ProfileShowcase";
import type { ProfileTab } from "#/types/generated/ProfileTab";
import type { StatusOk } from "#/types/generated/StatusOk";

/**
 * A save of the profile layout: only the keys sent change, the rest are kept as stored.
 * `tabs` replaces the tab list, `showcase` the blocks, `background` the header art
 * (`null` removes it).
 */
export interface IProfileLayoutPatch {
    tabs?: ProfileTab[];
    showcase?: ProfileShowcase;
    background?: ProfileBackground | null;
}

export interface IUpdateUserSettingsInput {
    public_profile: boolean;
    store_gacha: boolean;
    share_stats: boolean;
    /**
     * Omitted leaves the saved layout alone, `null` resets it to the default (tabs,
     * showcase and background alike), a patch sets the keys it carries (the backend
     * normalizes them before storing).
     */
    profile_layout?: IProfileLayoutPatch | null;
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

/** Any subset of the settings: a field left out stays as stored. */
export const updateUserSettingsFn = createServerFn({ method: "POST" })
    .inputValidator((data: Partial<IUpdateUserSettingsInput>) => data)
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
