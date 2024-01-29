import { redis } from "..";
import type { AKServer } from "../../lib/impl/authentication/auth";
import { leaderboard } from "../../lib/impl/database/impl/leaderboard";
import { type LeaderboardType } from "../../types/types";
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

        const server = (body?.server ?? paths[1] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        const type = body?.type ?? paths[2] ?? url.searchParams.get("type") ?? "level";
        if (type !== "level" && type !== "trust") {
            return createResponse(JSON.stringify({ error: "Invalid leaderboard type given." }));
        }

        const sort = body?.sort ?? paths[3] ?? url.searchParams.get("sort") ?? "desc";
        if (sort !== "asc" && sort !== "desc") {
            return createResponse(JSON.stringify({ error: "Invalid sort type given." }));
        }

        const limit = body?.limit ?? paths[4] ?? url.searchParams.get("limit") ?? 20;
        if (isNaN(Number(limit))) {
            return createResponse(JSON.stringify({ error: "Invalid limit given." }));
        }

        let fields: string[] = body?.fields ?? [];
        const fieldsParam = url.searchParams.get("fields");

        if (fieldsParam && fieldsParam.startsWith("[") && fieldsParam.endsWith("]")) {
            const fieldsArray = fieldsParam
                .slice(1, -1)
                .split(",")
                .map((field) => field.trim());
            fields = fieldsArray.filter(Boolean);
        }

        try {
            const data = await leaderboard({ server, type, sort, limit: Number(limit), fields });

            await redis.set(`leaderboard-${server}-${type}-${sort}-${limit}-${JSON.stringify(fields)}`, JSON.stringify(data), "EX", route.cacheTime);
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
    path: "/leaderboard",
    handler,
    rateLimit: 40,
    cacheTime: 60 * 60 * 24 * 7,
};

type Body = {
    server: AKServer;
    type?: LeaderboardType;
    sort?: "asc" | "desc";
    limit?: number;
    fields?: string[];
};

export default route;
