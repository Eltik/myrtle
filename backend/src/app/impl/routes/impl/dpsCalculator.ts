import { OperatorData } from "../../../../lib/impl/dps-calculator/impl/classes/impl/operator-data";
import operators from "../../../../lib/impl/local/impl/gamedata/impl/operators";
import { OperatorParams } from "../../../../types/impl/lib/impl/dps-calculator";
import middleware from "../../middleware";
import operatorsList from "../../../../lib/impl/dps-calculator/impl/operators";

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

        const id = body?.id ?? paths[1] ?? url.searchParams.get("id") ?? null;
        if (!id) {
            return middleware.createResponse(JSON.stringify({ error: "No character ID provided." }), 400);
        }
        const params = body?.params ?? {};
        const enemy = body?.enemy ?? {};

        const operator = operators(id);
        if (!operator) {
            return middleware.createResponse(JSON.stringify({ error: "Operator not found." }), 404);
        }

        try {
            const operatorsUnit = operatorsList.find((op) => op.id === operator.id);
            if (!operatorsUnit) {
                return middleware.createResponse(JSON.stringify({ error: "Operator not found. Their DPS calculations might not be added yet." }), 404);
            }
            const operatorData = new operatorsUnit.object(new OperatorData(operator), params);
            const data = operatorData.skillDPS({
                defense: enemy.defense ?? 0,
                res: enemy.res ?? 0,
            });
            return middleware.createResponse(
                JSON.stringify({
                    dps: data,
                    operator: operatorData,
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
    path: "/dps-calculator",
    handler,
    rateLimit: 20,
};

type Body = {
    id: string;
    params?: OperatorParams;
    enemy?: {
        defense?: number;
        res?: number;
    };
};

export default route;
