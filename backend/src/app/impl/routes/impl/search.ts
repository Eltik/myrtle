import { db } from "../../../../database";
import { tableName as usersTableName } from "../../../../database/impl/users";
import { JsonPath } from "../../../../types/impl/database";
import { UserDB } from "../../../../types/impl/database/impl/users";
import { AKServer } from "../../../../types/impl/lib/impl/authentication";
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

        const nickname = body?.nickname ?? paths[1] ?? url.searchParams.get("nickname") ?? null;
        if (!nickname) {
            return middleware.createResponse(JSON.stringify({ error: "No nickname provided." }), 400);
        }

        const nicknumber = body?.nicknumber ?? paths[2] ?? url.searchParams.get("nicknumber") ?? null;

        const server = (body?.server ?? paths[3] ?? url.searchParams.get("server") ?? "en") as AKServer;
        if (server && !["en", "jp", "kr", "cn", "bili", "tw"].includes(server)) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid server given." }), 400);
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
            const jsonConditions = [
                {
                    column: "data" as Extract<keyof UserDB, string>,
                    path: ["status", "nickName"] as JsonPath<UserDB[keyof UserDB]>,
                    operator: "LIKE" as "=" | "LIKE",
                    value: `%${nickname}%`,
                },
            ];

            if (nicknumber) {
                jsonConditions.push({
                    column: "data",
                    path: ["status", "nickNumber"],
                    operator: "=",
                    value: nicknumber,
                });
            }
            const data = await db.search<UserDB>(usersTableName, {
                conditions: {},
                jsonConditions: jsonConditions,
            }, fields);
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
    path: "/search",
    handler,
    rateLimit: 40,
    cacheTime: 60 * 60 * 24 * 7,
};

type Body = {
    nickname: string;
    nicknumber?: string;
    server?: AKServer;

    fields?: string[];
};

export default route;
