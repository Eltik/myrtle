import { parse, type SerializeOptions, serialize } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { env } from "~/env";

/**
 * Valid Arknights server regions
 */
export const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);
export type AKServer = z.infer<typeof AKServerSchema>;

/**
 * Default cookie options for auth cookies
 */
export const AUTH_COOKIE_OPTIONS: SerializeOptions = {
    httpOnly: true,
    secure: env.NODE_ENV === "production",
    sameSite: "strict",
    path: "/",
    maxAge: 60 * 60 * 24 * 7,
};

/**
 * Get JWT token from site_token cookie
 */
export function getToken(req: NextApiRequest): string | null {
    const cookies = parse(req.headers.cookie ?? "");
    return cookies.site_token ?? null;
}

/**
 * Set auth cookies after login
 * Stores JWT token in httpOnly cookie and sets client-visible indicator
 */
export function setAuthCookies(res: NextApiResponse, token: string): void {
    res.setHeader("Set-Cookie", [
        serialize("site_token", token, AUTH_COOKIE_OPTIONS),
        serialize("auth_indicator", "1", {
            ...AUTH_COOKIE_OPTIONS,
            httpOnly: false,
        }),
    ]);
}

/**
 * Clear auth cookies on logout or invalid session
 */
export function clearAuthCookies(res: NextApiResponse): void {
    const clearOptions: SerializeOptions = {
        ...AUTH_COOKIE_OPTIONS,
        maxAge: 0,
    };

    res.setHeader("Set-Cookie", [
        serialize("site_token", "", clearOptions),
        serialize("auth_indicator", "", { ...clearOptions, httpOnly: false }),
    ]);
}
