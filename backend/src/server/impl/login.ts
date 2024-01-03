import type { AKServer } from "../../lib/impl/authentication/auth";
import { loginWithEmailCode } from "../../lib/impl/authentication/auth-distributors/yostar";
import { AuthSession } from "../../lib/impl/authentication/auth-session";
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
        const code = body?.code ?? paths[2] ?? url.searchParams.get("code") ?? null;
        if (!code) {
            return createResponse(JSON.stringify({ error: "No code provided." }), 400);
        }
        const server = (body?.server ?? paths[3] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        const session = new AuthSession();

        try {
            const data = await loginWithEmailCode(server, session, email, code);
            return createResponse(
                JSON.stringify({
                    channelUID: data[0],
                    token: data[1],
                    uid: session.uid,
                    secret: session.secret,
                    seqnum: session.seqnum,
                }),
            );
        } catch (e: any) {
            return createResponse(e.message, 500);
        }
    } catch (e) {
        console.error(e);
        return createResponse(JSON.stringify({ error: "An error occurred." }), 500);
    }
};

const route = {
    path: "/login",
    handler,
    rateLimit: 20,
};

type Body = {
    email: string;
    code: string;
    server: AKServer;
};

export default route;
