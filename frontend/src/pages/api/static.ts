import { unstable_cache } from "next/cache";
import { env } from "~/env";
import type { Item } from "~/types/api/impl/material";
import type { Module, ModuleData, Modules } from "~/types/api/impl/module";
import type { Operator } from "~/types/api/impl/operator";
import type { Ranges } from "~/types/api/impl/range";
import type { Skill } from "~/types/api/impl/skill";
import type { Skin, SkinData } from "~/types/api/impl/skin";
import type { Voice, Voices } from "~/types/api/impl/voice";

import type { NextApiRequest, NextApiResponse } from "next";

// Define specific module response structures
interface ModulesListResponse {
    modules: Module[];
}
interface ModuleDetailsResponse {
    details: ModuleData;
}
interface SingleModuleResponse {
    modules: Module;
}
interface AllModulesResponse {
    modules: Modules;
}

type ModulesApiResponse = ModulesListResponse | ModuleDetailsResponse | SingleModuleResponse | AllModulesResponse;

interface RecruitmentTag {
    id: string;
    name: string;
    type: string;
}

interface OperatorOutcome {
    name: string;
    rarity: number;
    guaranteed: boolean;
    tags: string[];
}

// Assumed structure from backend's getRecruitment()
interface BackendRecruitmentData {
    TAG_MAP: Record<string, RecruitmentTag>;
    RECRUIT_POOL: Record<string, Operator>;
}

// Cache configuration
const CACHE_TAG = "static-api";
const CACHE_TTL = 3600; // Cache lifetime in seconds (1 hour)

// Determine if we're in development mode
const isDevelopment = env.NODE_ENV === "development";

// Function to fetch data from backend without caching
const fetchWithoutCache = async <T>(endpoint: string, body: object): Promise<T> => {
    const response = await fetch(`${env.BACKEND_URL}${endpoint}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        console.error(`Backend request failed: ${response.status} ${response.statusText}`);
        const errorBody = await response.text();
        console.error(`Backend error body: ${errorBody}`);
        throw new Error(`Backend request failed with status ${response.status}`);
    }

    return response.json() as Promise<T>;
};

// Function that decides whether to use cache or not based on environment
const fetchData = async <T>(endpoint: string, body: object, cacheKey?: string): Promise<T> => {
    // In development mode, always bypass cache
    if (isDevelopment) {
        console.log(`[DEV MODE] Fetching ${endpoint} without cache for body:`, body);
        return fetchWithoutCache<T>(endpoint, body);
    }

    // In production, use cache if a cache key is provided
    if (cacheKey) {
        console.log(`[PROD MODE] Attempting fetch with cache key: ${cacheKey}`);
        const cachedFetch = unstable_cache(async () => fetchWithoutCache<T>(endpoint, body), [cacheKey], { tags: [CACHE_TAG, cacheKey], revalidate: CACHE_TTL });
        return cachedFetch();
    }

    // Default fallback to non-cached fetch if no cache key specified in production
    console.log(`[PROD MODE] Fetching ${endpoint} without cache (no key) for body:`, body);
    return fetchWithoutCache<T>(endpoint, body);
};

// Request body type definition
interface RequestBody {
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook" | "skins" | "voices" | "gacha";
    id?: string;
    method?: string;
    trust?: number;
    fields?: string[];
    tags?: string[];
    recruitment?: string;
}

// Use NextApiRequest and NextApiResponse for Pages Router
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
    }

    try {
        // Body is already parsed by Next.js in Pages Router
        const body = req.body as RequestBody;

        switch (body.type) {
            case "materials": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-materials-${body.id ?? "all"}`;
                const materials = await fetchData<{ materials: Item[] }>("/static", requestBody, cacheKey);

                return res.status(200).json({ data: materials.materials });
            }
            case "modules": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                    method: body.method,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-modules-${body.id ?? "all"}-${body.method ?? "default"}`;
                const modules = await fetchData<ModulesApiResponse>("/static", requestBody, cacheKey);

                return res.status(200).json(modules);
            }
            case "operators": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                    fields: body.fields,
                };

                const fieldsKey = body.fields ? [...body.fields].sort().join(",") : "all";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-operators-${body.id ?? "all"}-fields-${fieldsKey}`;
                const operators = await fetchData<{ operators: Partial<Operator>[] | Partial<Operator> }>("/static", requestBody, cacheKey);

                return res.status(200).json({ data: operators.operators });
            }
            case "ranges": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-ranges-${body.id ?? "all"}`;
                const ranges = await fetchData<{ range: Ranges }>("/static", requestBody, cacheKey);

                return res.status(200).json({ data: ranges.range });
            }
            case "skills": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skills-${body.id ?? "all"}`;
                const skills = await fetchData<{ skills: Skill[] }>("/static", requestBody, cacheKey);

                return res.status(200).json({ data: skills.skills });
            }
            case "trust": {
                const requestBody = {
                    type: body.type,
                    trust: body.trust,
                };
                const cacheKey = body.trust === undefined && !isDevelopment ? `${CACHE_TAG}-trust-base` : undefined;
                const trust = await fetchData<{ trust: number | null }>("/static", requestBody, cacheKey);
                return res.status(200).json({ data: trust.trust });
            }
            case "handbook": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-handbook-${body.id ?? "all"}`;
                const handbook = await fetchData<{ handbook: unknown }>("/static", requestBody, cacheKey);
                return res.status(200).json({ data: handbook.handbook });
            }
            case "skins": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skins-${body.id ?? "all"}`;
                const skins = await fetchData<{ skins: Skin[] | SkinData }>("/static", requestBody, cacheKey);

                return res.status(200).json(skins);
            }
            case "voices": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                const voices = await fetchWithoutCache<{ voices: Voice[] | Voices }>("/static", requestBody);
                return res.status(200).json(voices);
            }
            case "gacha": {
                if (body.method === "recruitment") {
                    const requestBody = {
                        type: "gacha",
                        method: "recruitment",
                    };
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-recruitment-tags`;

                    const gachaData = await fetchData<{ recruitment: BackendRecruitmentData }>("/static", requestBody, cacheKey);

                    if (!gachaData.recruitment?.TAG_MAP) {
                        console.error("Invalid recruitment data structure received from backend:", gachaData);
                        throw new Error("Could not retrieve recruitment tags from backend.");
                    }

                    const tagsArray = Object.values(gachaData.recruitment.TAG_MAP);

                    return res.status(200).json({ data: tagsArray });
                } else if (body.method === "calculate") {
                    if (!body.tags || !Array.isArray(body.tags)) {
                        return res.status(400).json({ error: "Missing or invalid 'tags' array in request body." });
                    }

                    const recruitmentString = [...body.tags].sort().join(",");
                    const requestBody = {
                        type: "gacha",
                        method: "calculate",
                        recruitment: recruitmentString,
                    };
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-calculate-${recruitmentString}`;
                    const calcResult = await fetchData<{ recruitment: OperatorOutcome[] }>("/static", requestBody, cacheKey);

                    if (!calcResult.recruitment) {
                        console.error("Invalid calculation result structure received from backend:", calcResult);
                        throw new Error("Could not retrieve calculation results from backend.");
                    }
                    return res.status(200).json({ data: calcResult.recruitment });
                } else {
                    return res.status(400).json({ error: "Invalid 'method' for type 'gacha'. Use 'recruitment' or 'calculate'." });
                }
            }
            default: {
                const unknownType = body.type as string;
                console.warn(`Received unknown request type: ${unknownType}`);
                return res.status(400).json({ error: "Invalid type." });
            }
        }
    } catch (error: unknown) {
        console.error("API error in /api/static:", error);

        let statusCode = 500;
        let errorMessage = "An error occurred while processing the request.";

        if (error instanceof Error) {
            const backendErrorRegex = /^Backend request failed with status (\d+): (.+)$/;
            const httpErrorRegex = /^HTTP error (\d+): (.+)$/;

            const backendErrorMatch = backendErrorRegex.exec(error.message);
            const httpErrorMatch = httpErrorRegex.exec(error.message);

            if (backendErrorMatch) {
                statusCode = parseInt(backendErrorMatch[1] ?? "500", 10);
                errorMessage = backendErrorMatch[2] ?? (isDevelopment ? error.message : "Backend request failed.");
            } else if (httpErrorMatch) {
                statusCode = parseInt(httpErrorMatch[1] ?? "500", 10);
                errorMessage = httpErrorMatch[2] ?? (isDevelopment ? error.message : "An error occurred.");
            } else if (isDevelopment) {
                errorMessage = error.message;
            }
        }

        if (statusCode < 100 || statusCode > 599) {
            console.warn(`Corrected invalid status code ${statusCode} to 500.`);
            statusCode = 500;
        }

        return res.status(statusCode).json({ error: errorMessage });
    }
}
