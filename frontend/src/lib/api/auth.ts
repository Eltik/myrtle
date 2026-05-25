import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { backendFetch } from "#/lib/fetch";

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
        return (await res.json()) as { status: string };
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
