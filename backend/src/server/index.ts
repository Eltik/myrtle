import Redis from "ioredis";

import colors from "colors";

import { env } from "../env.ts";
import { rateLimitMiddleware } from "./lib/rateLimit.ts";
import { createResponse } from "./lib/response.ts";

export const redis: Redis = env.REDIS_URL
    ? new Redis(env.REDIS_URL || "redis://localhost:6379")
    : ({
          get: async () => null,
          set: (): Promise<"OK"> => Promise.resolve("OK"),
          on: () => Redis.prototype,
          keys: async () => [],
          connect: async () => void 0,
          call: async () => void 0,
      } as unknown as Redis);

export const start = async () => {
    const routes: {
        [key: string]: { path: string; handler: (req: Request) => Promise<Response>; rateLimit: number };
    } = {};
    const routeFiles = [
        await import("./impl/stats.ts"),
        await import("./impl/sendCode.ts"),
        await import("./impl/login.ts"),
        await import("./impl/refresh.ts"),
        await import("./impl/leaderboard.ts"),
        await import("./impl/searchPlayers.ts"),
        await import("./impl/player.ts"),
        await import("./impl/search.ts"),
    ];

    for (const file of routeFiles) {
        const routeModule = await file;
        const route = routeModule.default;

        if (route) {
            const { path, handler, rateLimit } = route;
            routes[path] = { path, handler, rateLimit };
        }
    }

    console.log(colors.gray(`Loaded ${colors.yellow(Object.keys(routes).length + "")} routes`));

    Bun.serve({
        port: env.PORT,
        async fetch(req: Request) {
            const url = new URL(req.url);
            if (url.pathname === "/") return createResponse("Welcome to Surtr API! 🎉", 200, { "Content-Type": "text/plain" });

            const pathName = `/${url.pathname.split("/").slice(1)[0]}`;

            if (routes[pathName]) {
                const { handler, rateLimit } = routes[pathName];
                const requests = await rateLimitMiddleware(req, pathName);

                if (requests && requests.requests > rateLimit) {
                    // Will only log up to 10 times
                    if (requests.requests > rateLimit * 2 && requests.requests < rateLimit * 2 + 10) console.log(colors.red(`Rate limit significantly exceeded for ${requests.ip} - ${pathName}`));

                    return createResponse(JSON.stringify({ error: "Too many requests" }), 429);
                }

                return handler(req);
            }

            return createResponse(JSON.stringify({ error: "Route not found" }), 404);
        },
    });

    console.log(colors.blue(`Server is now listening on ${env.PORT}`));
};
