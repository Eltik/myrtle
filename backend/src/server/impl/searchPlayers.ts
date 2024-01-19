import { cacheTime, redis } from "..";
import type { AKServer } from "../../lib/impl/authentication/auth";
import { AuthSession } from "../../lib/impl/authentication/auth-session";
import { searchPlayers } from "../../lib/impl/client";
import { createResponse } from "../lib/response";

export const handler = async (req: Request): Promise<Response> => {
    try {
        const url = new URL(req.url);
        const paths = url.pathname.split("/");
        paths.shift();

        const body =
            req.method === "POST"
                ? ((await req.json().catch(() => {
                      return null;
                  })) as Body)
                : null;

        const uid = body?.uid ?? paths[1] ?? url.searchParams.get("uid") ?? null;
        if (!uid) {
            return createResponse(JSON.stringify({ error: "No UID provided. You can use the /login route to obtain account details." }), 400);
        }
        const secret = body?.secret ?? paths[2] ?? url.searchParams.get("secret") ?? null;
        if (!secret) {
            return createResponse(JSON.stringify({ error: "No secret provided. You can use the /login route to obtain account details." }), 400);
        }
        const seqnum = isNaN(Number(body?.seqnum ?? paths[3] ?? url.searchParams.get("seqnum") ?? null)) ? undefined : Number(body?.seqnum ?? paths[3] ?? url.searchParams.get("seqnum") ?? null);
        if (!seqnum) {
            return createResponse(JSON.stringify({ error: "No seqnum provided. You can use the /login route to obtain account details." }), 400);
        }

        const nickname = body?.nickname ?? url.searchParams.get("nickname") ?? null;
        if (!nickname) {
            return createResponse(JSON.stringify({ error: "No nickname provided" }), 400);
        }
        const nicknumber = body?.nicknumber ?? url.searchParams.get("nicknumber") ?? null;
        const limit = isNaN(Number(body?.limit ?? url.searchParams.get("limit") ?? null)) ? undefined : Number(body?.limit ?? url.searchParams.get("limit") ?? null);

        const server = (body?.server ?? paths[4] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        const cached = await redis.get(`search-players-${server}-${nickname}-${nicknumber}-${limit}`);
        if (cached) {
            return createResponse(cached);
        }

        const session = new AuthSession(uid, secret, seqnum);

        try {
            const data = await searchPlayers(session, server, nickname, nicknumber ?? undefined, limit ?? undefined);
            await redis.set(`search-players-${server}-${nickname}-${nicknumber}-${limit}`, JSON.stringify(data), "EX", cacheTime);
            return createResponse(JSON.stringify(data));
        } catch (e: any) {
            return createResponse(e.message, 500);
        }
    } catch (e) {
        console.error(e);
        return createResponse(JSON.stringify({ error: "An error occurred." }), 500);
    }
};

const route = {
    path: "/search-players",
    handler,
    rateLimit: 20,
};

type Body = {
    uid: string;
    secret: string;
    seqnum: number;

    nickname: string;
    nicknumber: string;
    limit: number;

    server: AKServer;
};

export default route;
