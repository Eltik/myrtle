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

        const method = body?.method ?? paths[1] ?? url.searchParams.get("method") ?? null;

        if (!method) {
            return middleware.createResponse(JSON.stringify({ error: "No method provided." }), 400);
        }

        if (!["operator", "dps"].includes(method)) {
            return middleware.createResponse(JSON.stringify({ error: "Invalid method provided." }), 400);
        }

        const id = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

        if (method === "operator") {
            if (!id) {
                const operatorData = operatorsList.map((op) => {
                    const operator = operators(op.id);
                    if (!operator) {
                        return null;
                    }
                    return new op.object(new OperatorData(operator), {});
                });
                return middleware.createResponse(JSON.stringify({ operators: operatorData.filter(Boolean) }), 200);
            }

            const operator = operators(id);
            if (!operator) {
                return middleware.createResponse(JSON.stringify({ error: "Operator not found." }), 404);
            }

            const operatorsUnit = operatorsList.find((op) => op.id === operator.id);
            if (!operatorsUnit) {
                return middleware.createResponse(JSON.stringify({ error: "Operator not found. Their DPS calculations might not be added yet." }), 404);
            }

            const operatorData = new operatorsUnit.object(new OperatorData(operator), {});

            return middleware.createResponse(JSON.stringify({ operator: operatorData }), 200);
        }

        if (!id) {
            return middleware.createResponse(JSON.stringify({ error: "No character ID provided." }), 400);
        }

        const params = body?.params ?? {};
        const range = body?.range ?? {};

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

            const dpsData = {
                def: [],
                res: [],
            } as {
                def: {
                    dps: number;
                    def: number;
                }[];
                res: {
                    dps: number;
                    res: number;
                }[];
            };
            for (let def = range.minDef ?? 0; def <= (range.maxDef ?? 10000); def += 100) {
                for (let res = range.minRes ?? 0; res <= (range.maxRes ?? 10000); res += 100) {
                    const data = operatorData.skillDPS({
                        defense: def,
                        res: res,
                    });
                    dpsData.def.push({
                        dps: data,
                        def: def,
                    });
                    dpsData.res.push({
                        dps: data,
                        res: res,
                    });
                }

                dpsData.def.sort((a, b) => a.dps - b.dps);
                dpsData.res.sort((a, b) => a.dps - b.dps);
            }

            const totalDPS = dpsData.def.reduce((acc, curr) => acc + curr.dps, 0);
            const averageDPS = totalDPS / dpsData.def.length;

            return middleware.createResponse(
                JSON.stringify({
                    dps: dpsData,
                    operator: operatorData,
                    totalDPS,
                    averageDPS,
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
    rateLimit: 500,
};

type Body = {
    method: "operator" | "dps";
    id?: string;
    params?: OperatorParams;
    range?: {
        minDef?: number;
        maxDef?: number;
        minRes?: number;
        maxRes?: number;
    };
};

export default route;
