import type Module from "node:module";
// Import necessary types for recruitment - adjust paths/definitions as needed
import type { NextApiRequest, NextApiResponse } from "next";
import { unstable_cache } from "next/cache";
import { env } from "~/env.js";
import type { Item } from "~/types/impl/api/static/material";
import type { ModuleData, Modules } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Ranges } from "~/types/impl/api/static/ranges";
import type { Skill } from "~/types/impl/api/static/skills";
import type { Skin, SkinData } from "~/types/impl/api/static/skins";
import type { Voice, Voices } from "~/types/impl/api/static/voices";

// Define specific module response structures if known, otherwise keep broad
// Example structures (adjust based on actual backend responses):
interface ModulesListResponse {
    modules: Module[];
} // Assuming Module is imported or defined
interface ModuleDetailsResponse {
    details: ModuleData;
} // Assuming ModuleData is imported or defined
interface SingleModuleResponse {
    modules: Module;
}
interface AllModulesResponse {
    modules: Modules;
} // Assuming Modules is imported or defined

type ModulesApiResponse = ModulesListResponse | ModuleDetailsResponse | SingleModuleResponse | AllModulesResponse;

interface RecruitmentTag {
    id: string;
    name: string;
    type: string; // Or specific types: 'Qualification' | 'Position' | 'Profession' | 'Affix'
}

interface OperatorOutcome {
    name: string;
    rarity: number;
    guaranteed: boolean;
    tags: string[];
}

// Assumed structure from backend's getRecruitment()
interface BackendRecruitmentData {
    TAG_MAP: Record<string, RecruitmentTag>; // Assuming TAG_MAP holds the tag objects
    RECRUIT_POOL: Record<string, Operator>; // Define more accurately if possible
    // ... other properties
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
        // Add cache: 'no-store' to explicitly bypass HTTP caching if needed
        // cache: 'no-store',
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
        // unstable_cache expects the key parts as an array
        console.log(`[PROD MODE] Attempting fetch with cache key: ${cacheKey}`);
        // Directly using fetchWithCache requires passing the key parts array to it.
        // Let's call unstable_cache directly here for clarity, ensuring key uniqueness.
        const cachedFetch = unstable_cache(
            async () => fetchWithoutCache<T>(endpoint, body),
            [cacheKey], // Key MUST be string[]
            { tags: [CACHE_TAG, cacheKey], revalidate: CACHE_TTL },
        );
        return cachedFetch();

        // --- Alternative: Passing key to fetchWithCache (if its signature is adapted) ---
        // return fetchWithCache<T>(endpoint, body, cacheKey); // Requires fetchWithCache's key param to be string
    }

    // Default fallback to non-cached fetch if no cache key specified in production
    console.log(`[PROD MODE] Fetching ${endpoint} without cache (no key) for body:`, body);
    return fetchWithoutCache<T>(endpoint, body);
};

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
                // Use the defined union type
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
            // Trust and Handbook might not be needed for recruitment calc, keep original logic
            case "trust": {
                const requestBody = {
                    type: body.type,
                    // Backend expects 'trust' value, not 'id'
                    trust: body.trust,
                };
                // Cache only if trust value isn't provided (implies fetching base value? Check backend logic)
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
                // Use unknown for handbook data type
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

                return res.status(200).json(skins); // Return structure as received
            }
            case "voices": {
                const requestBody = {
                    type: body.type,
                    id: body.id,
                };
                // Caching was previously commented out, keeping it that way unless needed
                // const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-voices-${body.id ?? "all"}`;
                const voices = await fetchWithoutCache<{ voices: Voice[] | Voices }>("/static", requestBody);
                return res.status(200).json(voices); // Return structure as received
            }
            // --- START: Gacha Handling (with fixes) ---
            case "gacha": {
                if (body.method === "recruitment") {
                    // Fetch all recruitment tags
                    const requestBody = {
                        type: "gacha",
                        method: "recruitment",
                    };
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-recruitment-tags`;

                    // Expect backend to return { recruitment: { TAG_MAP: { ... } } }
                    const gachaData = await fetchData<{ recruitment: BackendRecruitmentData }>("/static", requestBody, cacheKey);

                    // Use optional chaining
                    if (!gachaData.recruitment?.TAG_MAP) {
                        console.error("Invalid recruitment data structure received from backend:", gachaData);
                        throw new Error("Could not retrieve recruitment tags from backend.");
                    }

                    // Extract tags from the TAG_MAP object
                    const tagsArray = Object.values(gachaData.recruitment.TAG_MAP);

                    return res.status(200).json({ data: tagsArray }); // Return just the array of tags
                } else if (body.method === "calculate") {
                    if (!body.tags || !Array.isArray(body.tags) /* Removed length check here as backend handles empty */) {
                        // Keep basic client-side validation
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
                        throw new Error("Could not retrieve calculation results from backend."); // Let generic catch handle
                    }
                    return res.status(200).json({ data: calcResult.recruitment });
                } else {
                    return res.status(400).json({ error: "Invalid 'method' for type 'gacha'. Use 'recruitment' or 'calculate'." });
                }
            }
            // --- END: Gacha Handling ---
            default: {
                // Cast body.type to string for logging
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
            // Regex definitions
            const backendErrorRegex = /^Backend request failed with status (\d+): (.+)$/;
            const httpErrorRegex = /^HTTP error (\d+): (.+)$/;

            // Use exec() instead of match()
            const backendErrorMatch = backendErrorRegex.exec(error.message);
            const httpErrorMatch = httpErrorRegex.exec(error.message);

            if (backendErrorMatch) {
                // Provide default '500' to parseInt if group 1 is undefined
                statusCode = parseInt(backendErrorMatch[1] ?? "500", 10);
                // Use nullish coalescing for error message
                errorMessage = backendErrorMatch[2] ?? (isDevelopment ? error.message : "Backend request failed.");
            } else if (httpErrorMatch) {
                // Provide default '500' to parseInt if group 1 is undefined
                statusCode = parseInt(httpErrorMatch[1] ?? "500", 10);
                // Use nullish coalescing for error message
                errorMessage = httpErrorMatch[2] ?? (isDevelopment ? error.message : "An error occurred.");
            } else if (isDevelopment) {
                errorMessage = error.message;
            }
        }

        // Ensure status code is valid (redundant check for parseInt default, but safe)
        if (statusCode < 100 || statusCode > 599) {
            console.warn(`Corrected invalid status code ${statusCode} to 500.`);
            statusCode = 500;
        }

        return res.status(statusCode).json({ error: errorMessage });
    }
}

// Update RequestBody type
interface RequestBody {
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook" | "skins" | "voices" | "gacha";
    id?: string;
    method?: string; // Now used for 'modules' and 'gacha'
    trust?: number;
    fields?: string[];
    tags?: string[]; // Added: Expected from client for calculation
    recruitment?: string; // Added: Sent to backend for calculation
}
