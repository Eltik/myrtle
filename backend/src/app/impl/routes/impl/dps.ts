import { redis } from "../../..";
import { env } from "../../../../env";
import { calculateDPS } from "../../../../lib/impl/dps-calculator";
import operators from "../../../../lib/impl/local/impl/gamedata/impl/operators";
import type { CalculateDPSParams } from "../../../../types/impl/lib/impl/dps-calculator";
import middleware from "../../middleware";

const handler = async (req: Request): Promise<Response> => {
    try {
        const url = new URL(req.url);
        const paths = url.pathname.split("/");
        paths.shift();

        if (req.method !== "POST") {
            return middleware.createResponse(JSON.stringify({ error: "This route only supports the POST method." }), 405);
        }

        const body =
            ((await req.json().catch(() => {
                return null;
            })) as Body) ?? null;

        if (!body) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid request body." }), 400);
        }

        const { charId, options, operatorData, enemy, buffConfig } = body;

        try {
            const cached = await redis.get(`dps:${charId}:${JSON.stringify(options)}:${JSON.stringify(operatorData)}:${JSON.stringify(enemy)}:${JSON.stringify(buffConfig)}`);
            if (cached) {
                return middleware.createResponse(cached);
            }

            const operator = await operators(charId);
            if (!operator) {
                return middleware.createResponse(JSON.stringify({ error: "Operator not found for given ID." }), 400);
            }

            const data = calculateDPS({
                char: operator,
                options,
                operatorData,
                enemy,
                buffConfig,
            });

            await redis.set(`dps:${charId}:${JSON.stringify(options)}:${JSON.stringify(operatorData)}:${JSON.stringify(enemy)}:${JSON.stringify(buffConfig)}`, JSON.stringify(data), "EX", env.REDIS_CACHE_TIME);

            return middleware.createResponse(
                JSON.stringify({
                    ...data,
                }),
            );
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
    path: "/dps",
    handler,
    rateLimit: 20,
};

type Body = {
    charId: string;
    options: CalculateDPSParams["options"];
    operatorData: CalculateDPSParams["operatorData"];
    enemy: CalculateDPSParams["enemy"];
    buffConfig: CalculateDPSParams["buffConfig"];
};

export default route;
