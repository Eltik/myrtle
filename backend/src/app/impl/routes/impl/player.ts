import { db } from "../../../../database";
import { tableName as userTableName } from "../../../../database/impl/users";
import { UserDB } from "../../../../types/impl/database/impl/users";
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

        const uid = body?.uid ?? paths[1] ?? url.searchParams.get("uid") ?? null;
        if (!uid) {
            return middleware.createResponse(JSON.stringify({ error: "No UID provided." }), 400);
        }

        const server = (body?.server ?? paths[2] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        try {
            const data = await db.read<UserDB>(userTableName, {
                uid,
                server,
            });
            return middleware.createResponse(JSON.stringify(data));
        } catch (e) {
            return middleware.createResponse((e as { message: string }).message, 500);
        }
    } catch (e) {
        console.error(e);
        return middleware.createResponse(JSON.stringify({ error: "An error occurred." }), 500);
    }
};

const route = {
    path: "/player",
    handler,
    rateLimit: 50,
};

type Body = {
    uid: string;
    server: AKServer;
};

export default route;