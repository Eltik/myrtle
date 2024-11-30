import { db } from "../../../../database";
import { tableName as userTableName } from "../../../../database/impl/users";
import user from "../../../../lib/impl/user";
import type { UserDB } from "../../../../types/impl/database/impl/users";
import { AuthSession, type AKServer } from "../../../../types/impl/lib/impl/authentication";
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
            return middleware.createResponse(JSON.stringify({ error: "No UID provided. You can use the /login route to obtain account details." }), 400);
        }
        const secret = body?.secret ?? paths[2] ?? url.searchParams.get("secret") ?? null;
        if (!secret) {
            return middleware.createResponse(JSON.stringify({ error: "No secret provided. You can use the /login route to obtain account details." }), 400);
        }
        const seqnum = isNaN(Number(body?.seqnum ?? paths[3] ?? url.searchParams.get("seqnum") ?? null)) ? undefined : Number(body?.seqnum ?? paths[3] ?? url.searchParams.get("seqnum") ?? null);
        if (!seqnum) {
            return middleware.createResponse(JSON.stringify({ error: "No seqnum provided. You can use the /login route to obtain account details." }), 400);
        }

        const server = (body?.server ?? paths[4] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (!["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
        }

        const session = new AuthSession(uid, secret, Number(seqnum));

        try {
            const data = await user.get(session, server);
            const stored = await db.read<UserDB>(userTableName, {
                uid: uid,
                server: server,
            });

            if (!stored || stored.length === 0) {
                const length = 16;
                const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let result = "";
                for (let i = 0; i < length; i++) {
                    const randomIndex = Math.floor(Math.random() * chars.length);
                    result += chars[randomIndex];
                }

                if (data)
                    await db.create<UserDB>(userTableName, {
                        id: result,
                        uid: uid,
                        data: data,
                        server: server,
                        created_at: new Date(Date.now()).toISOString(),
                    });
            } else {
                if (data)
                    await db.update<UserDB>(
                        userTableName,
                        {
                            uid: uid,
                            server: server,
                        },
                        {
                            data: data,
                        },
                    );
            }

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
    path: "/refresh",
    handler,
    rateLimit: 20,
};

type Body = {
    uid: string;
    secret: string;
    seqnum: number;

    server: AKServer;
};

export default route;
