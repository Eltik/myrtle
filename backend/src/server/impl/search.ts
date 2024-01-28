import type { AKServer } from "../../lib/impl/authentication/auth";
import { search } from "../../lib/impl/database/impl/search";
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

        const nickname = body?.nickname ?? paths[1] ?? url.searchParams.get("nickname") ?? null;
        if (!nickname) {
            return createResponse(JSON.stringify({ error: "No nickname provided." }), 400);
        }

        const nicknumber = body?.nicknumber ?? paths[2] ?? url.searchParams.get("nicknumber") ?? null;

        const server = (body?.server ?? paths[3] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (server && !["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        try {
            const data = await search({ nickname, nicknumber, server });
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
    path: "/search",
    handler,
    rateLimit: 40,
    cacheTime: 60 * 60 * 24 * 7
};

type Body = {
    nickname: string;
    nicknumber?: string;
    server?: AKServer;
};

export default route;
