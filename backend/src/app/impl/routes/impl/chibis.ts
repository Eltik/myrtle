import chibi, { getAll as getAllChibis } from "../../../../lib/impl/local/impl/gamedata/impl/chibi";
import { extractOperatorList, processCharsForFrontend } from "../../../../lib/impl/local/impl/gamedata/impl/chibi/impl/process";

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

        // Extract query parameters
        const chibisId = body?.id ?? paths[1] ?? url.searchParams.get("id") ?? null;
        const format = body?.format ?? url.searchParams.get("format") ?? "full";

        // If an ID is provided, return a specific chibi
        if (chibisId) {
            const chibisData = chibi(chibisId);
            if (!chibisData) {
                return new Response(JSON.stringify({ error: "Chibi not found" }), {
                    status: 404,
                    headers: {
                        "Content-Type": "application/json",
                    },
                });
            }
            return new Response(JSON.stringify(chibisData), {
                headers: {
                    "Content-Type": "application/json",
                },
            });
        }

        // Return all chibis with optional formatting
        const allChibis = getAllChibis();

        if (format === "simplified") {
            // Create a simplified structure that's easier for the frontend to work with
            const simplifiedChibis = processCharsForFrontend(allChibis);
            return new Response(JSON.stringify(simplifiedChibis), {
                headers: {
                    "Content-Type": "application/json",
                },
            });
        } else if (format === "operatorList") {
            // Return just a list of operator codes (e.g. "char_002_amiya")
            const operatorList = extractOperatorList(allChibis);
            return new Response(JSON.stringify(operatorList), {
                headers: {
                    "Content-Type": "application/json",
                },
            });
        } else {
            // Return full data
            return new Response(JSON.stringify(allChibis), {
                headers: {
                    "Content-Type": "application/json",
                },
            });
        }
    } catch (error) {
        console.error("Error in chibis route:", error);
        return new Response(JSON.stringify({ error: "Internal Server Error" }), {
            status: 500,
            headers: {
                "Content-Type": "application/json",
            },
        });
    }
};

const route = {
    path: "/chibis",
    handler,
    rateLimit: 30,
    cacheTime: 60 * 60 * 24, // 1 day cache
};

export type Body = {
    id?: string;
    format?: "full" | "simplified" | "operatorList";
};

export default route;
