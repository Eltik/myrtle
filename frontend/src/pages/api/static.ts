import type { NextApiRequest, NextApiResponse } from "next";
import { unstable_cache } from "next/cache";
import { env } from "~/env";
import { backendFetch } from "~/lib/backend-fetch";
import type { ChibiCharacter } from "~/types/api/impl/chibi";
import type { Enemy, EnemyInfoList, RaceData } from "~/types/api/impl/enemy";
import type { Item } from "~/types/api/impl/material";
import type { Module, ModuleData, Modules } from "~/types/api/impl/module";
import type { Operator } from "~/types/api/impl/operator";
import type { Ranges } from "~/types/api/impl/range";
import type { Skill } from "~/types/api/impl/skill";
import type { Skin, SkinData } from "~/types/api/impl/skin";
import type { Voice, Voices } from "~/types/api/impl/voice";

interface ModulesListResponse {
    modules: Module[];
}
interface ModuleDetailsResponse {
    details: ModuleData;
}
interface SingleModuleResponse {
    module: Module;
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
    id: string;
    name: string;
    rarity: number;
    profession: string;
    position: string;
    guaranteed: boolean;
    tags: string[];
}

interface BackendGachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

interface BackendRecruitmentResponse {
    recruitment: {
        tags: BackendGachaTag[];
        tagMap: Record<string, BackendGachaTag>;
        tagNameMap: Record<string, BackendGachaTag>;
        recruitDetail: string;
        recruitPool: unknown;
    };
}

const TAG_GROUP_TYPE_MAP: Record<number, string> = {
    0: "Qualification",
    1: "Position",
    2: "Class",
    3: "Affix",
};

const RARITY_TIER_MAP: Record<string, number> = {
    TIER_6: 6,
    TIER_5: 5,
    TIER_4: 4,
    TIER_3: 3,
    TIER_2: 2,
    TIER_1: 1,
};

interface BackendOperator {
    id: string;
    name: string;
    rarity: string;
    profession: string;
    position: string;
    tag_list?: string[];
}

interface BackendCalculateResponse {
    recruitment: Array<{
        label: string[];
        operators: BackendOperator[];
    }>;
}

const CACHE_TAG = "static-api";
const CACHE_TTL = 3600;

const isDevelopment = env.NODE_ENV === "development";

const fetchWithoutCache = async <T>(endpoint: string): Promise<T> => {
    if (isDevelopment) {
        console.log(`[DEV MODE] Fetching GET ${env.BACKEND_URL}${endpoint}`);
    }

    const response = await backendFetch(endpoint, {
        method: "GET",
        headers: {
            "Accept-Encoding": "gzip, deflate",
        },
    });

    if (!response.ok) {
        console.error(`Backend request failed: ${response.status} ${response.statusText}`);
        const errorBody = await response.text();
        console.error(`Backend error body: ${errorBody}`);
        throw new Error(`Backend request failed with status ${response.status}`);
    }

    return response.json() as Promise<T>;
};

const fetchData = async <T>(endpoint: string, cacheKey?: string): Promise<T> => {
    if (isDevelopment) {
        return fetchWithoutCache<T>(endpoint);
    }

    if (cacheKey) {
        console.log(`[PROD MODE] Attempting fetch with cache key: ${cacheKey}`);
        const cachedFetch = unstable_cache(async () => fetchWithoutCache<T>(endpoint), [cacheKey], { tags: [CACHE_TAG, cacheKey], revalidate: CACHE_TTL });
        return cachedFetch();
    }

    console.log(`[PROD MODE] Fetching ${endpoint} without cache (no key)`);
    return fetchWithoutCache<T>(endpoint);
};

interface RequestBody {
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook" | "skins" | "voices" | "gacha" | "chibis" | "enemies" | "enemy" | "enemyRaces" | "enemyLevelInfo";
    id?: string;
    method?: string;
    trust?: number;
    fields?: string[];
    tags?: string[];
    recruitment?: string;
    limit?: number;
    cursor?: string;
    level?: number;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
    }

    try {
        const body = req.body as RequestBody;

        switch (body.type) {
            case "materials": {
                const endpoint = body.id ? `/static/materials/${body.id}` : "/static/materials";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-materials-${body.id ?? "all"}`;
                const materials = await fetchData<{ materials?: Item[]; material?: Item }>(endpoint, cacheKey);

                const data = materials.material ? materials.material : materials.materials;
                return res.status(200).json({ data });
            }
            case "modules": {
                let endpoint: string;
                if (body.method === "details" && body.id) {
                    endpoint = `/static/modules/details/${body.id}`;
                } else if (body.id) {
                    endpoint = `/static/modules/${body.id}`;
                } else {
                    endpoint = "/static/modules";
                }
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-modules-${body.id ?? "all"}-${body.method ?? "default"}`;
                const modules = await fetchData<ModulesApiResponse>(endpoint, cacheKey);

                return res.status(200).json(modules);
            }
            case "operators": {
                let endpoint = body.id ? `/static/operators/${body.id}` : "/static/operators";
                const queryParams: string[] = [];

                const limit = body.limit ?? 1000;
                queryParams.push(`limit=${limit}`);

                if (body.fields && body.fields.length > 0) {
                    const fieldsParam = body.fields.join(",");
                    queryParams.push(`fields=${encodeURIComponent(fieldsParam)}`);
                }

                if (queryParams.length > 0) {
                    endpoint += `?${queryParams.join("&")}`;
                }

                const fieldsKey = body.fields ? [...body.fields].sort().join(",") : "all";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-operators-${body.id ?? "all"}-fields-${fieldsKey}-limit-${limit}`;
                const operators = await fetchData<{ operators?: Partial<Operator>[]; operator?: Partial<Operator> }>(endpoint, cacheKey);

                const data = operators.operator ? operators.operator : operators.operators;
                return res.status(200).json({ data });
            }
            case "ranges": {
                const endpoint = body.id ? `/static/ranges/${body.id}` : "/static/ranges";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-ranges-${body.id ?? "all"}`;
                const ranges = await fetchData<{ range?: Ranges; ranges?: Ranges[] }>(endpoint, cacheKey);

                const data = ranges.range ? ranges.range : ranges.ranges;
                return res.status(200).json({ data });
            }
            case "skills": {
                const endpoint = body.id ? `/static/skills/${body.id}` : "/static/skills";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skills-${body.id ?? "all"}`;
                const skills = await fetchData<{ skills?: Skill[]; skill?: Skill }>(endpoint, cacheKey);

                const data = skills.skill ? skills.skill : skills.skills;
                return res.status(200).json({ data });
            }
            case "trust": {
                let endpoint = "/static/trust";
                if (body.trust !== undefined) {
                    endpoint = `/static/trust/calculate?trust=${body.trust}`;
                }
                const cacheKey = body.trust === undefined && !isDevelopment ? `${CACHE_TAG}-trust-base` : undefined;
                const trust = await fetchData<{ trust?: number; level?: number; favor?: unknown }>(endpoint, cacheKey);

                const _data = trust.level !== undefined ? trust.level : trust.favor;
                return res.status(200).json({ data: trust });
            }
            case "handbook": {
                const endpoint = body.id ? `/static/handbook/${body.id}` : "/static/handbook";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-handbook-${body.id ?? "all"}`;
                const handbook = await fetchData<{ handbook: unknown }>(endpoint, cacheKey);
                return res.status(200).json({ data: handbook.handbook });
            }
            case "skins": {
                let endpoint: string;
                if (body.id) {
                    if (body.id.startsWith("char_")) {
                        endpoint = `/static/skins/char/${body.id}`;
                    } else {
                        endpoint = `/static/skins/${body.id}`;
                    }
                } else {
                    endpoint = "/static/skins";
                }
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skins-${body.id ?? "all"}`;
                const skins = await fetchData<{ skins: Skin[] | SkinData }>(endpoint, cacheKey);

                return res.status(200).json(skins);
            }
            case "voices": {
                if (body.method === "voice-actors") {
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-voice-actors-map`;
                    const allVoices = await fetchData<{ voices: Voice[]; total: number; has_more: boolean }>("/static/voices?limit=20000", cacheKey);
                    const voiceActors: Record<string, string[]> = {};
                    for (const voice of allVoices.voices) {
                        if (!voice.charId) continue;
                        if (!voiceActors[voice.charId]) voiceActors[voice.charId] = [];
                        const existing = new Set(voiceActors[voice.charId]);
                        for (const d of voice.data ?? []) {
                            for (const name of d.cvName ?? []) {
                                if (name && !existing.has(name)) {
                                    existing.add(name);
                                    voiceActors[voice.charId]?.push(name);
                                }
                            }
                        }
                    }
                    return res.status(200).json({ voiceActors });
                }

                let endpoint: string;
                if (body.id) {
                    if (body.id.startsWith("char_")) {
                        endpoint = `/static/voices/char/${body.id}`;
                    } else {
                        endpoint = `/static/voices/${body.id}`;
                    }
                } else {
                    endpoint = "/static/voices";
                }
                const voices = await fetchData<{ voices: Voice[] | Voices }>(endpoint);
                return res.status(200).json(voices);
            }
            case "gacha": {
                if (body.method === "recruitment") {
                    const endpoint = "/static/gacha/recruitment";
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-recruitment-tags`;

                    const gachaData = await fetchData<BackendRecruitmentResponse>(endpoint, cacheKey);

                    if (!gachaData.recruitment?.tags || !Array.isArray(gachaData.recruitment.tags)) {
                        console.error("Invalid recruitment data structure received from backend:", gachaData);
                        throw new Error("Could not retrieve recruitment tags from backend.");
                    }

                    const tagsArray: RecruitmentTag[] = gachaData.recruitment.tags.map((tag) => ({
                        id: String(tag.tagId),
                        name: tag.tagName,
                        type: TAG_GROUP_TYPE_MAP[tag.tagGroup] ?? "Affix",
                    }));

                    return res.status(200).json({ data: tagsArray });
                } else if (body.method === "calculate") {
                    if (!body.tags || !Array.isArray(body.tags)) {
                        return res.status(400).json({ error: "Missing or invalid 'tags' array in request body." });
                    }

                    const recruitmentString = [...body.tags].sort().join(",");
                    const endpoint = `/static/gacha/calculate?recruitment=${encodeURIComponent(recruitmentString)}`;
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-calculate-${recruitmentString}`;
                    const calcResult = await fetchData<BackendCalculateResponse>(endpoint, cacheKey);

                    if (!calcResult.recruitment || !Array.isArray(calcResult.recruitment)) {
                        console.error("Invalid calculation result structure received from backend:", calcResult);
                        throw new Error("Could not retrieve calculation results from backend.");
                    }

                    const operatorOutcomes: OperatorOutcome[] = [];

                    for (const group of calcResult.recruitment) {
                        const hasTopOperator = group.label.includes("Top Operator");
                        const hasSeniorOperator = group.label.includes("Senior Operator");
                        const _minRarity = hasTopOperator ? 6 : hasSeniorOperator ? 5 : 3;

                        for (const op of group.operators) {
                            const rarity = RARITY_TIER_MAP[op.rarity] ?? 1;
                            const allHighRarity = group.operators.every((o) => (RARITY_TIER_MAP[o.rarity] ?? 1) >= 4);

                            operatorOutcomes.push({
                                id: op.id,
                                name: op.name,
                                rarity,
                                profession: op.profession,
                                position: op.position,
                                guaranteed: hasTopOperator || hasSeniorOperator || allHighRarity,
                                tags: body.tags,
                            });
                        }
                    }

                    return res.status(200).json({ data: operatorOutcomes });
                } else {
                    return res.status(400).json({ error: "Invalid 'method' for type 'gacha'. Use 'recruitment' or 'calculate'." });
                }
            }
            case "chibis": {
                let endpoint: string;
                if (body.id) {
                    if (body.id.startsWith("char_")) {
                        endpoint = `/static/chibis/${body.id}`;
                    } else {
                        endpoint = `/static/chibis/${body.id}`;
                    }
                } else {
                    endpoint = "/static/chibis";
                }
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-chibis-${body.id ?? "all"}`;
                const chibis = await fetchData<{ chibi?: ChibiCharacter; chibis?: ChibiCharacter[] }>(endpoint, cacheKey);
                return res.status(200).json(chibis);
            }
            case "enemies": {
                const queryParams: string[] = [];
                if (body.cursor) queryParams.push(`cursor=${encodeURIComponent(body.cursor)}`);
                if (body.limit) queryParams.push(`limit=${body.limit}`);
                if (body.fields && body.fields.length > 0) queryParams.push(`fields=${encodeURIComponent(body.fields.join(","))}`);
                if (body.level !== undefined) queryParams.push(`level=${body.level}`);

                const endpoint = queryParams.length > 0 ? `/static/enemies?${queryParams.join("&")}` : "/static/enemies";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemies-${body.cursor ?? "start"}-${body.limit ?? "default"}-${body.level ?? "all"}`;
                const enemies = await fetchData<{ enemies: Enemy[]; hasMore: boolean; nextCursor: string | null; total: number }>(endpoint, cacheKey);
                return res.status(200).json(enemies);
            }
            case "enemy": {
                if (!body.id) {
                    return res.status(400).json({ error: "Missing 'id' for enemy request." });
                }
                const endpoint = `/static/enemies/${body.id}`;
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemy-${body.id}`;
                const enemy = await fetchData<{ enemy: Enemy }>(endpoint, cacheKey);
                return res.status(200).json(enemy);
            }
            case "enemyRaces": {
                const endpoint = "/static/enemies/races";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemy-races`;
                const races = await fetchData<{ races: Record<string, RaceData> }>(endpoint, cacheKey);
                return res.status(200).json(races);
            }
            case "enemyLevelInfo": {
                const endpoint = "/static/enemies/levels";
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemy-level-info`;
                const levelInfo = await fetchData<{ levels: EnemyInfoList[] }>(endpoint, cacheKey);
                return res.status(200).json(levelInfo);
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
