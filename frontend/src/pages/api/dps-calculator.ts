import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    try {
        switch (req.method) {
            case "GET": {
                const response = await backendFetch("/dps/operators");

                if (!response.ok) {
                    return res.status(response.status).json({ error: "Failed to fetch DPS operators" });
                }

                const data = await response.json();
                return res.status(200).json(data);
            }

            case "POST": {
                const body = req.body as {
                    operatorId: string;
                    params?: Record<string, unknown>;
                    enemy?: { defense?: number; res?: number };
                    range?: {
                        minDef?: number;
                        maxDef?: number;
                        defStep?: number;
                        minRes?: number;
                        maxRes?: number;
                        resStep?: number;
                    };
                };

                // Shared flat request body (without defense/res — those are swept or fixed per call)
                const baseBody: Record<string, unknown> = {
                    operatorId: body.operatorId,
                    ...(body.params ?? {}),
                };

                // Helper to call v3 /dps/calculate with a specific (defense, res) pair
                const callV3 = async (defense: number, resVal: number): Promise<number> => {
                    const r = await backendFetch("/dps/calculate", {
                        method: "POST",
                        body: JSON.stringify({ ...baseBody, defense, res: resVal }),
                    });
                    if (!r.ok) {
                        const errText = await r.text().catch(() => "");
                        throw new Error(`v3 DPS error ${r.status}: ${errText}`);
                    }
                    const result = (await r.json()) as { skill_dps: number; total_damage: number; average_dps: number };
                    return result.average_dps;
                };

                try {
                    // RANGE mode: build DPS curves by iterating DEF and RES independently
                    if (body.range) {
                        const minDef = body.range.minDef ?? 0;
                        const maxDef = body.range.maxDef ?? 3000;
                        const defStep = body.range.defStep ?? 100;
                        const minRes = body.range.minRes ?? 0;
                        const maxRes = body.range.maxRes ?? 120;
                        const resStep = body.range.resStep ?? 10;

                        // Build DEF sweep at RES=0
                        const defCalls: Array<Promise<{ value: number; dps: number }>> = [];
                        for (let d = minDef; d <= maxDef; d += defStep) {
                            defCalls.push(callV3(d, 0).then((dps) => ({ value: d, dps })));
                        }

                        // Build RES sweep at DEF=0
                        const resCalls: Array<Promise<{ value: number; dps: number }>> = [];
                        for (let rv = minRes; rv <= maxRes; rv += resStep) {
                            resCalls.push(callV3(0, rv).then((dps) => ({ value: rv, dps })));
                        }

                        const [byDefense, byResistance] = await Promise.all([
                            Promise.all(defCalls),
                            Promise.all(resCalls),
                        ]);

                        return res.status(200).json({
                            dps: { byDefense, byResistance },
                            operator: { id: body.operatorId, name: "", rarity: 0, profession: "" },
                        });
                    }

                    // SINGLE-POINT mode
                    const defense = body.enemy?.defense ?? 0;
                    const resVal = body.enemy?.res ?? 0;

                    const r = await backendFetch("/dps/calculate", {
                        method: "POST",
                        body: JSON.stringify({ ...baseBody, defense, res: resVal }),
                    });
                    if (!r.ok) {
                        const errText = await r.text().catch(() => "");
                        return res.status(r.status).json({ error: "Failed to calculate DPS", details: errText });
                    }
                    const full = (await r.json()) as { skill_dps: number; total_damage: number; average_dps: number };

                    return res.status(200).json({
                        dps: {
                            skillDps: full.skill_dps,
                            totalDamage: full.total_damage,
                            averageDps: full.average_dps,
                        },
                        operator: { id: body.operatorId, name: "", rarity: 0, profession: "" },
                    });
                } catch (err) {
                    const msg = err instanceof Error ? err.message : String(err);
                    console.error(`[DPS Calculate] ${msg}`);
                    return res.status(500).json({ error: "Failed to calculate DPS", details: msg });
                }
            }

            default: {
                res.setHeader("Allow", ["GET", "POST"]);
                return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
            }
        }
    } catch (error) {
        console.error("[DPS Calculator] API error:", error);
        return res.status(500).json({ error: "An error occurred while processing the request." });
    }
}
