import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";
import type { DisconnectResult } from "#/types/generated/DisconnectResult";
import type { StatusOk } from "#/types/generated/StatusOk";

export interface IUpdateUserSettingsInput {
    public_profile: boolean;
    store_gacha: boolean;
    share_stats: boolean;
}

export const updateUserSettingsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpdateUserSettingsInput) => data)
    .handler(async ({ data }) => {
        const token = getCookie("site_token");
        if (!token) throw new Error("Not signed in.");
        const res = await backendFetch("/auth/update-settings", {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify(data),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to update settings: ${res.status}`);
        }
        return (await res.json()) as StatusOk;
    });

/**
 * Forget the stored Yostar credentials for the signed-in account.
 *
 * Leaves the site session and the already-synced data alone - it only revokes
 * our ability to reach Yostar. `removed` reports whether anything was stored.
 */
export const disconnectGameAccountFn = createServerFn({ method: "POST" }).handler(async (): Promise<{ removed: boolean }> => {
    const token = getCookie("site_token");
    if (!token) throw new Error("Not signed in.");
    const res = await backendFetch("/auth/disconnect", {
        method: "POST",
        bearerToken: token,
    });
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || `Failed to disconnect: ${res.status}`);
    }
    // `removed` is required on the generated type, so the `?? false` this used
    // to carry is gone: the backend always sends it.
    const body = (await res.json()) as DisconnectResult;
    return { removed: body.removed };
});

export const refreshRosterFn = createServerFn({ method: "POST" }).handler(async (): Promise<{ status: string }> => {
    const token = getCookie("site_token");
    if (!token) throw new Error("Not signed in.");
    const res = await backendFetch("/refresh", {
        method: "POST",
        bearerToken: token,
    });
    if (!res.ok) {
        const text = await res.text().catch(() => "");
        throw new Error(text || `Failed to re-sync roster: ${res.status}`);
    }
    return { status: "ok" };
});
