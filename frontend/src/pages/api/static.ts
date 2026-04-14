import type { NextApiRequest, NextApiResponse } from "next";
import { unstable_cache } from "next/cache";
import { env } from "~/env";
import { backendFetch } from "~/lib/backend-fetch";
import type { ChibiCharacter } from "~/types/api/impl/chibi";
import type { Enemy, EnemyHandbook } from "~/types/api/impl/enemy";
import type { GachaData, GachaTag } from "~/types/api/impl/gacha";
import type { Item } from "~/types/api/impl/material";
import type { Module, Modules } from "~/types/api/impl/module";
import type { Operator } from "~/types/api/impl/operator";
import type { Ranges } from "~/types/api/impl/range";
import type { Skill } from "~/types/api/impl/skill";
import type { Skin, SkinData } from "~/types/api/impl/skin";
import type { Voice, VoiceLang, VoiceLangDictEntry, Voices } from "~/types/api/impl/voice";

interface RecruitmentTag {
    id: string;
    name: string;
    type: string;
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
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-materials`;
                const materials = await fetchData<Record<string, Item>>("/static/materials", cacheKey);

                if (body.id) {
                    const item = materials[body.id];
                    return res.status(200).json({ data: item ?? null });
                }
                return res.status(200).json({ data: Object.values(materials) });
            }
            case "modules": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-modules`;
                const modules = await fetchData<Modules>("/static/modules", cacheKey);

                if (body.method === "details" && body.id) {
                    // Look up module details by ID from the full modules data
                    // The Modules type contains the full module dataset; extract what's needed
                    return res.status(200).json({ details: (modules as unknown as Record<string, unknown>)[body.id] ?? null });
                }
                if (body.id) {
                    // Look up a specific module by operator ID or module ID
                    const found = (modules as unknown as Record<string, unknown>)[body.id] ?? null;
                    return res.status(200).json({ module: found });
                }
                return res.status(200).json({ modules });
            }
            case "operators": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-operators`;
                const operators = await fetchData<Record<string, Operator>>("/static/operators", cacheKey);

                if (body.id) {
                    const op = operators[body.id];
                    return res.status(200).json({ data: op ?? null });
                }

                let data: Partial<Operator>[] = Object.values(operators);

                // If fields were requested, filter to only those fields for backward compat
                if (body.fields && body.fields.length > 0) {
                    const fields = body.fields;
                    data = data.map((op) => {
                        const filtered: Record<string, unknown> = {};
                        const opRecord = op as unknown as Record<string, unknown>;
                        for (const field of fields) {
                            if (field in opRecord) {
                                filtered[field] = opRecord[field];
                            }
                        }
                        return filtered as Partial<Operator>;
                    });
                }

                return res.status(200).json({ data });
            }
            case "ranges": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-ranges`;
                const ranges = await fetchData<Record<string, Ranges>>("/static/ranges", cacheKey);

                if (body.id) {
                    const range = ranges[body.id];
                    return res.status(200).json({ data: range ?? null });
                }
                return res.status(200).json({ data: Object.values(ranges) });
            }
            case "skills": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skills`;
                const skills = await fetchData<Record<string, Skill>>("/static/skills", cacheKey);

                if (body.id) {
                    const skill = skills[body.id];
                    return res.status(200).json({ data: skill ?? null });
                }
                return res.status(200).json({ data: Object.values(skills) });
            }
            case "trust": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-trust`;
                const trust = await fetchData<unknown>("/static/trust", cacheKey);
                return res.status(200).json({ data: trust });
            }
            case "handbook": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-handbook`;
                const handbook = await fetchData<Record<string, unknown>>("/static/handbook", cacheKey);

                if (body.id) {
                    const entry = handbook[body.id];
                    return res.status(200).json({ data: entry ?? null });
                }
                return res.status(200).json({ data: handbook });
            }
            case "skins": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-skins`;
                const skins = await fetchData<SkinData>("/static/skins", cacheKey);

                if (body.id) {
                    if (body.id.startsWith("char_")) {
                        // Filter skins belonging to this character
                        const charSkins = Object.values(skins.charSkins ?? {}).filter((s: Skin) => s.charId === body.id);
                        return res.status(200).json({ skins: charSkins });
                    }
                    // Look up a specific skin by skin ID
                    const skin = skins.charSkins?.[body.id] ?? null;
                    return res.status(200).json({ skins: skin });
                }
                return res.status(200).json({ skins });
            }
            case "voices": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-voices`;
                const allVoices = await fetchData<Voices>("/static/voices", cacheKey);

                if (body.method === "voice-actors") {
                    // Build voice actor lookup from voiceLangDict
                    const voiceActors: Record<string, string[]> = {};
                    for (const [, langEntry] of Object.entries(allVoices.voiceLangDict ?? {})) {
                        const entry = langEntry as VoiceLang;
                        if (!entry.charId) continue;
                        if (!voiceActors[entry.charId]) voiceActors[entry.charId] = [];
                        const existing = new Set(voiceActors[entry.charId]);
                        for (const [, dictVal] of Object.entries(entry.dict ?? {}) as [string, VoiceLangDictEntry][]) {
                            for (const name of dictVal.cvName ?? []) {
                                if (name && !existing.has(name)) {
                                    existing.add(name);
                                    voiceActors[entry.charId]?.push(name);
                                }
                            }
                        }
                    }
                    return res.status(200).json({ voiceActors });
                }

                if (body.id) {
                    if (body.id.startsWith("char_")) {
                        // Filter voices for this character from charWords
                        const charVoices = Object.values(allVoices.charWords ?? {}).filter((v: Voice) => v.charId === body.id);
                        // Also get the voiceLangDict entry for this character
                        const langEntries: Record<string, VoiceLang> = {};
                        for (const [key, entry] of Object.entries(allVoices.voiceLangDict ?? {})) {
                            if ((entry as VoiceLang).charId === body.id) {
                                langEntries[key] = entry as VoiceLang;
                            }
                        }
                        return res.status(200).json({ voices: charVoices, voiceLangDict: langEntries });
                    }
                    // Look up a specific voice by ID
                    const voice = allVoices.charWords?.[body.id] ?? null;
                    return res.status(200).json({ voices: voice });
                }
                return res.status(200).json({ voices: allVoices });
            }
            case "gacha": {
                if (body.method === "recruitment") {
                    const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-gacha-recruitment-tags`;
                    const gachaData = await fetchData<GachaData>("/static/gacha", cacheKey);

                    if (!gachaData.gachaTags || !Array.isArray(gachaData.gachaTags)) {
                        console.error("Invalid gacha data structure - no gachaTags:", gachaData);
                        throw new Error("Could not retrieve recruitment tags from backend.");
                    }

                    const tagsArray: RecruitmentTag[] = gachaData.gachaTags.map((tag: GachaTag) => ({
                        id: String(tag.tagId),
                        name: tag.tagName,
                        type: TAG_GROUP_TYPE_MAP[tag.tagGroup] ?? "Affix",
                    }));

                    return res.status(200).json({ data: tagsArray });
                } else if (body.method === "calculate") {
                    // v3 backend does not have a /static/gacha/calculate endpoint.
                    // Return the full gacha data so the client can calculate locally,
                    // or return an error indicating client-side calculation is needed.
                    if (!body.tags || !Array.isArray(body.tags)) {
                        return res.status(400).json({ error: "Missing or invalid 'tags' array in request body." });
                    }

                    // Fetch operators and gacha data, then do server-side calculation
                    const [gachaData, operators] = await Promise.all([
                        fetchData<GachaData>("/static/gacha", isDevelopment ? undefined : `${CACHE_TAG}-gacha`),
                        fetchData<Record<string, Operator>>("/static/operators", isDevelopment ? undefined : `${CACHE_TAG}-operators`),
                    ]);

                    // Build tag name -> tag mapping
                    const tagNameMap: Record<string, GachaTag> = {};
                    for (const tag of gachaData.gachaTags ?? []) {
                        tagNameMap[tag.tagName] = tag;
                    }

                    // Find recruitable operators (those with tags in the recruit pool)
                    // Filter operators that have recruitment tags
                    const recruitableOps = Object.values(operators).filter((op) => {
                        return op.tagList && op.tagList.length > 0;
                    });

                    // For each selected tag combination, find matching operators
                    const selectedTags = body.tags;

                    // Map tag names to their categories for matching
                    const matchingOperators: Array<{
                        id: string;
                        name: string;
                        rarity: number;
                        profession: string;
                        position: string;
                        guaranteed: boolean;
                        tags: string[];
                    }> = [];

                    const hasTopOperator = selectedTags.includes("Top Operator");
                    const hasSeniorOperator = selectedTags.includes("Senior Operator");

                    for (const op of recruitableOps) {
                        const opRarity = RARITY_TIER_MAP[op.rarity] ?? 1;

                        // Build the operator's full tag set (including position, class, qualification tags)
                        const opTags = new Set<string>(op.tagList ?? []);
                        if (op.position) opTags.add(op.position === "MELEE" ? "Melee" : op.position === "RANGED" ? "Ranged" : op.position);
                        if (op.profession) {
                            const professionTagMap: Record<string, string> = {
                                PIONEER: "Vanguard",
                                WARRIOR: "Guard",
                                TANK: "Defender",
                                SNIPER: "Sniper",
                                CASTER: "Caster",
                                SUPPORT: "Supporter",
                                MEDIC: "Medic",
                                SPECIAL: "Specialist",
                            };
                            const classTag = professionTagMap[op.profession];
                            if (classTag) opTags.add(classTag);
                        }
                        // Add qualification tags based on rarity
                        if (opRarity === 6) opTags.add("Top Operator");
                        if (opRarity >= 5) opTags.add("Senior Operator");
                        if (opRarity === 1) opTags.add("Robot");
                        if (opRarity <= 2) opTags.add("Starter");

                        // Check if operator matches ALL selected tags
                        const matchesAll = selectedTags.every((tag) => opTags.has(tag));
                        if (!matchesAll) continue;

                        // Filter by rarity rules: without Top/Senior, 6★ are excluded from recruit
                        if (!hasTopOperator && opRarity === 6) continue;
                        if (!hasSeniorOperator && !hasTopOperator && opRarity === 5) continue;

                        matchingOperators.push({
                            id: op.id ?? "",
                            name: op.name ?? "",
                            rarity: opRarity,
                            profession: op.profession ?? "",
                            position: op.position ?? "",
                            guaranteed: hasTopOperator || hasSeniorOperator,
                            tags: selectedTags,
                        });
                    }

                    // Recalculate guaranteed flag: if all results are 4★+, it's guaranteed
                    const allHighRarity = matchingOperators.length > 0 && matchingOperators.every((op) => op.rarity >= 4);
                    if (allHighRarity) {
                        for (const op of matchingOperators) {
                            op.guaranteed = true;
                        }
                    }

                    return res.status(200).json({ data: matchingOperators });
                } else {
                    return res.status(400).json({ error: "Invalid 'method' for type 'gacha'. Use 'recruitment' or 'calculate'." });
                }
            }
            case "chibis": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-chibis`;
                // v3 returns { characters: ChibiCharacter[] } (by_operator is skipped in serialization)
                const data = await fetchData<{ characters: ChibiCharacter[] }>("/static/chibis", cacheKey);
                const characters = data.characters ?? [];

                if (body.id) {
                    const chibi = characters.find((c) => c.operatorCode === body.id);
                    return res.status(200).json({ chibi: chibi ?? null });
                }
                return res.status(200).json({ chibis: characters });
            }
            case "enemies": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemies`;
                const enemyData = await fetchData<EnemyHandbook | Record<string, Enemy>>("/static/enemies", cacheKey);

                // v3 returns the full enemy dataset. It may be an EnemyHandbook structure
                // or a flat Record<string, Enemy>. Handle both.
                let enemies: Enemy[];
                if ("enemyData" in enemyData && typeof enemyData.enemyData === "object") {
                    enemies = Object.values((enemyData as EnemyHandbook).enemyData);
                } else {
                    enemies = Object.values(enemyData as Record<string, Enemy>);
                }

                // Apply level filter if specified
                if (body.level !== undefined) {
                    enemies = enemies.filter((e) => {
                        if (!e.stats?.levels) return false;
                        return e.stats.levels.some((l) => l.level === body.level);
                    });
                }

                // Apply pagination for backward compat
                const limit = body.limit ?? enemies.length;
                const cursorIndex = body.cursor ? parseInt(body.cursor, 10) : 0;
                const paginatedEnemies = enemies.slice(cursorIndex, cursorIndex + limit);
                const hasMore = cursorIndex + limit < enemies.length;
                const nextCursor = hasMore ? String(cursorIndex + limit) : null;

                return res.status(200).json({
                    enemies: paginatedEnemies,
                    hasMore,
                    nextCursor,
                    total: enemies.length,
                });
            }
            case "enemy": {
                if (!body.id) {
                    return res.status(400).json({ error: "Missing 'id' for enemy request." });
                }
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemies`;
                const enemyData = await fetchData<EnemyHandbook | Record<string, Enemy>>("/static/enemies", cacheKey);

                let enemy: Enemy | undefined;
                if ("enemyData" in enemyData && typeof enemyData.enemyData === "object") {
                    enemy = (enemyData as EnemyHandbook).enemyData[body.id];
                } else {
                    enemy = (enemyData as Record<string, Enemy>)[body.id];
                }

                if (!enemy) {
                    return res.status(404).json({ error: `Enemy '${body.id}' not found.` });
                }
                return res.status(200).json({ enemy });
            }
            case "enemyRaces": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemies`;
                const enemyData = await fetchData<EnemyHandbook | Record<string, unknown>>("/static/enemies", cacheKey);

                if ("raceData" in enemyData && typeof enemyData.raceData === "object") {
                    return res.status(200).json({ races: (enemyData as EnemyHandbook).raceData });
                }

                // If the data doesn't have raceData, try to extract race info from enemies
                console.warn("enemyRaces: raceData not found in /static/enemies response, returning empty.");
                return res.status(200).json({ races: {} });
            }
            case "enemyLevelInfo": {
                const cacheKey = isDevelopment ? undefined : `${CACHE_TAG}-enemies`;
                const enemyData = await fetchData<EnemyHandbook | Record<string, unknown>>("/static/enemies", cacheKey);

                if ("levelInfoList" in enemyData && Array.isArray(enemyData.levelInfoList)) {
                    return res.status(200).json({ levels: (enemyData as EnemyHandbook).levelInfoList });
                }

                console.warn("enemyLevelInfo: levelInfoList not found in /static/enemies response, returning empty.");
                return res.status(200).json({ levels: [] });
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
