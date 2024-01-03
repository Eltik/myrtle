import type { AKServer } from "../../lib/impl/authentication/auth";
import { requestYostarAuth } from "../../lib/impl/authentication/auth-distributors/yostar";
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

        const email = body?.email ?? paths[1] ?? url.searchParams.get("email") ?? null;
        if (!email) {
            return createResponse(JSON.stringify({ error: "No email provided." }), 400);
        }

        const server = (body?.server ?? paths[2] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        try {
            const data = await requestYostarAuth(email, server);
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
    path: "/send-code",
    handler,
    rateLimit: 20,
};

type Body = {
    email: string;
    server: AKServer;
};

export default route;
