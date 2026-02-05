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
 * Session cookie validation schema
 */
export const SessionSchema = z.object({
    uid: z.string().min(1, "UID is required"),
    secret: z.string().min(1, "Secret is required"),
    seqnum: z.number().int().min(0),
    server: AKServerSchema,
    yostar_email: z.string().optional(),
    yssid: z.string().optional(),
    yssid_sig: z.string().optional(),
});
export type SessionData = z.infer<typeof SessionSchema>;

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
 * Parse and validate full session from auth_session cookie
 */
export function getSessionFromCookie(req: NextApiRequest): SessionData | null {
    const cookies = parse(req.headers.cookie ?? "");
    const sessionCookie = cookies.auth_session;

    if (!sessionCookie) {
        return null;
    }

    try {
        const sessionData = JSON.parse(sessionCookie);
        const parseResult = SessionSchema.safeParse(sessionData);
        return parseResult.success ? parseResult.data : null;
    } catch {
        return null;
    }
}

/**
 * Get site token from cookie for backend authentication
 */
export function getSiteToken(req: NextApiRequest): string | null {
    const cookies = parse(req.headers.cookie ?? "");
    return cookies.site_token ?? null;
}

/**
 * Set auth cookies after login or refresh
 */
export function setAuthCookies(
    res: NextApiResponse,
    session: {
        uid: string;
        secret: string;
        seqnum: number;
        server: string;
        yostar_email?: string;
        yssid?: string;
        yssid_sig?: string;
    },
    siteToken: string,
): void {
    const sessionData = JSON.stringify(session);

    res.setHeader("Set-Cookie", [
        serialize("auth_session", sessionData, AUTH_COOKIE_OPTIONS),
        serialize("site_token", siteToken, AUTH_COOKIE_OPTIONS),
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

    res.setHeader("Set-Cookie", [serialize("auth_session", "", clearOptions), serialize("site_token", "", clearOptions), serialize("auth_indicator", "", { ...clearOptions, httpOnly: false })]);
}
