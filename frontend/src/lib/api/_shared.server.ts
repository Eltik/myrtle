import { getCookie } from "@tanstack/react-start/server";

import { ApiError } from "./_shared";

/** Throws `ApiError(401)` when the caller is unauthenticated. Server-side only. */
export function requireSiteToken(): string {
    const token = getCookie("site_token");
    if (!token) throw new ApiError(401, "Not signed in");
    return token;
}

export function optionalSiteToken(): string | undefined {
    return getCookie("site_token") || undefined;
}
