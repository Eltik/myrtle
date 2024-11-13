import { sendCode } from "../../../../lib/impl/authentication/impl/yostar";
import type { AKServer } from "../../../../types/impl/lib/impl/authentication";
import middleware from "../../middleware";

const handler = async (req: Request): Promise<Response> => {
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
            return middleware.createResponse(JSON.stringify({ error: "No email provided." }), 400);
        }

        const server = (body?.server ?? paths[2] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        try {
            const data = await sendCode(email, server);
            return middleware.createResponse(JSON.stringify(data));
        } catch (e) {
            console.error(e);
            return middleware.createResponse((e as { message: string }).message, 500);
        }
    } catch (e) {
        console.error(e);
        return middleware.createResponse(JSON.stringify({ error: "An error occurred." }), 500);
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
