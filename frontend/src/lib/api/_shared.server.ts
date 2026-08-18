import { getCookie } from "@tanstack/react-start/server";

import { APIError } from "./_shared";

/** Throws `APIError(401)` when the caller is unauthenticated. Server-side only. */
export function requireSiteToken(): string {
    const token = getCookie("site_token");
    if (!token) throw new APIError(401, "Not signed in");
    return token;
}

export function optionalSiteToken(): string | undefined {
    return getCookie("site_token") || undefined;
}
