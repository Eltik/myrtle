import { redis } from "..";
import { createResponse } from "../lib/response";

export const handler = async (req: Request): Promise<Response> => {
    try {
        const url = new URL(req.url);
        const paths = url.pathname.split("/");
        paths.shift();

        const cached = await redis.get(`stats`);
        if (cached) {
            return createResponse(cached);
        }

        const data = {};
        if (!data) {
            return createResponse(JSON.stringify({ error: "No data found." }), 404);
        }

        await redis.set(`stats`, JSON.stringify(data), "EX", route.cacheTime);

        return createResponse(JSON.stringify(data));
    } catch (e) {
        console.error(e);
        return createResponse(JSON.stringify({ error: "An error occurred." }), 500);
    }
};

const route = {
    path: "/stats",
    handler,
    rateLimit: 60,
    cacheTime: 60 * 60 * 24 * 7,
};

export default route;
