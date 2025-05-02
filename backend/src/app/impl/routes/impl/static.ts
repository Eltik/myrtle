import { redis, REDIS_KEY } from "../../..";
import { env } from "../../../../env";
import { filterObject } from "../../../../helper";
import { calculateTrust, getHandbook, getMaterial, modules } from "../../../../lib/impl/local/impl/gamedata";
import { getAll as getAllMaterials } from "../../../../lib/impl/local/impl/gamedata/impl/materials";
import { getAll as getAllModules } from "../../../../lib/impl/local/impl/gamedata/impl/modules";
import { getAll as getAllGacha } from "../../../../lib/impl/local/impl/gamedata/impl/gacha";
import operators, { getAll as getAllOperators } from "../../../../lib/impl/local/impl/gamedata/impl/operators";
import ranges, { getAll as getAllRanges } from "../../../../lib/impl/local/impl/gamedata/impl/ranges";
import skills, { getAll as getAllSkills } from "../../../../lib/impl/local/impl/gamedata/impl/skills";
import skins, { getAll as getAllSkins } from "../../../../lib/impl/local/impl/gamedata/impl/skins";
import voices, { getAll as getAllVoices } from "../../../../lib/impl/local/impl/gamedata/impl/voices";
import middleware from "../../middleware";
import { calculateRecruitment, getRecruitment } from "../../../../lib/impl/local/impl/gamedata/impl/gacha/impl/recruitment";

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

        const type = body?.type ?? paths[1] ?? url.searchParams.get("type") ?? null;
        if (!type) {
            return middleware.createResponse(JSON.stringify({ error: "No type provided." }), 400);
        }

        try {
            switch (type.toLowerCase()) {
                case "materials":
                    const materialId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const materialsCached = await redis.get(`${REDIS_KEY}-static:materials:${materialId ?? "none"}`);
                    if (materialsCached) {
                        return middleware.createResponse(materialsCached);
                    }

                    const materials = materialId ? getMaterial(materialId) : getAllMaterials();

                    await redis.set(
                        `${REDIS_KEY}-static:materials:${materialId ?? "none"}`,
                        JSON.stringify({
                            materials,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            materials,
                        }),
                    );
                case "modules":
                    const method = body?.method ?? paths[2] ?? url.searchParams.get("method") ?? null;
                    if (!method) {
                        const id = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;

                        const modulesIdCached = await redis.get(`${REDIS_KEY}-static:modules:${id ?? "none"}`);
                        if (modulesIdCached) {
                            return middleware.createResponse(modulesIdCached);
                        }

                        const modulesData = id ? modules.get(id) : await getAllModules();

                        await redis.set(
                            `${REDIS_KEY}-static:modules:${id ?? "none"}`,
                            JSON.stringify({
                                modules: modulesData,
                            }),
                            "EX",
                            env.REDIS_CACHE_TIME,
                        );

                        return middleware.createResponse(
                            JSON.stringify({
                                modules: modulesData,
                            }),
                        );
                    } else {
                        switch (method.toLowerCase()) {
                            case "charid":
                                const charId = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;

                                const modulesCharIdCached = await redis.get(`${REDIS_KEY}-static:modules:charid:${charId ?? "none"}`);
                                if (modulesCharIdCached) {
                                    return middleware.createResponse(modulesCharIdCached);
                                }

                                const modulesData = modules.getByCharId(charId);

                                await redis.set(
                                    `${REDIS_KEY}-static:modules:charid:${charId ?? "none"}`,
                                    JSON.stringify({
                                        modules: modulesData,
                                    }),
                                    "EX",
                                    env.REDIS_CACHE_TIME,
                                );

                                return middleware.createResponse(
                                    JSON.stringify({
                                        modules: modulesData,
                                    }),
                                );
                            case "details":
                                const id = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;

                                const modulesDetailsCached = await redis.get(`${REDIS_KEY}-static:modules:details:${id ?? "none"}`);
                                if (modulesDetailsCached) {
                                    return middleware.createResponse(modulesDetailsCached);
                                }

                                const details = modules.getModuleDetails(id);

                                await redis.set(
                                    `${REDIS_KEY}-static:modules:details:${id ?? "none"}`,
                                    JSON.stringify({
                                        details,
                                    }),
                                    "EX",
                                    env.REDIS_CACHE_TIME,
                                );

                                return middleware.createResponse(
                                    JSON.stringify({
                                        details,
                                    }),
                                );
                            default:
                                return middleware.createResponse(JSON.stringify({ error: "Invalid method." }), 400);
                        }
                    }
                case "operators":
                    const operatorId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;
                    const fieldsParam = body?.fields ?? url.searchParams.get("fields") ?? null;

                    let fields: string[] | null = null;
                    if (typeof fieldsParam === "string") {
                        // Split comma-separated string from query params
                        fields = fieldsParam
                            .split(",")
                            .map((f) => f.trim())
                            .filter((f) => f.length > 0);
                    } else if (Array.isArray(fieldsParam)) {
                        // Use array directly from body
                        fields = fieldsParam.filter((f) => typeof f === "string" && f.length > 0);
                    }
                    // Ensure fields is null if empty after processing or invalid type
                    if (!fields || fields.length === 0) {
                        fields = null;
                    }

                    const fieldsCacheKey = fields ? [...fields].sort().join(",") : "all";
                    const cacheKey = `${REDIS_KEY}-static:operators:${operatorId ?? "none"}:fields:${fieldsCacheKey}`;

                    const operatorsCached = await redis.get(cacheKey);
                    if (operatorsCached) {
                        return middleware.createResponse(operatorsCached);
                    }

                    const operatorsDataRaw = operatorId ? operators(operatorId) : getAllOperators();

                    let finalOperatorsData: any = operatorsDataRaw;
                    if (fields && operatorsDataRaw) {
                        if (Array.isArray(operatorsDataRaw)) {
                            finalOperatorsData = operatorsDataRaw.map((op: any) => filterObject(op, fields!));
                        } else {
                            finalOperatorsData = filterObject(operatorsDataRaw, fields);
                        }
                    }

                    await redis.set(
                        cacheKey,
                        JSON.stringify({
                            operators: finalOperatorsData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            operators: finalOperatorsData,
                        }),
                    );
                case "ranges":
                    const rangeId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const rangesCached = await redis.get(`${REDIS_KEY}-static:ranges:${rangeId ?? "none"}`);
                    if (rangesCached) {
                        return middleware.createResponse(rangesCached);
                    }

                    const range = rangeId ? ranges(rangeId) : getAllRanges();

                    await redis.set(
                        `${REDIS_KEY}-static:ranges:${rangeId ?? "none"}`,
                        JSON.stringify({
                            range,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            range,
                        }),
                    );
                case "skills":
                    const skillId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const skillsCached = await redis.get(`${REDIS_KEY}-static:skills:${skillId ?? "none"}`);
                    if (skillsCached) {
                        return middleware.createResponse(skillsCached);
                    }

                    const skillsData = skillId ? skills(skillId) : getAllSkills();

                    await redis.set(
                        `${REDIS_KEY}-static:skills:${skillId ?? "none"}`,
                        JSON.stringify({
                            skills: skillsData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            skills: skillsData,
                        }),
                    );
                case "trust":
                    const trust = body?.trust ?? paths[2] ?? url.searchParams.get("trust") ?? null;

                    const trustCached = await redis.get(`${REDIS_KEY}-static:trust:${trust ?? "none"}`);
                    if (trustCached) {
                        return middleware.createResponse(trustCached);
                    }

                    const trustData = trust ? calculateTrust(Number(trust)) : null;

                    await redis.set(
                        `${REDIS_KEY}-static:trust:${trust ?? "none"}`,
                        JSON.stringify({
                            trust: trustData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            trust: trustData,
                        }),
                    );
                case "handbook":
                    const charId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const handbookCached = await redis.get(`${REDIS_KEY}-static:handbook:${charId ?? "none"}`);
                    if (handbookCached) {
                        return middleware.createResponse(handbookCached);
                    }

                    const handbookData = charId ? getHandbook(charId) : null;

                    await redis.set(
                        `${REDIS_KEY}-static:handbook:${charId ?? "none"}`,
                        JSON.stringify({
                            handbook: handbookData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            handbook: handbookData,
                        }),
                    );
                case "skins":
                    const skinId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const skinsCached = await redis.get(`${REDIS_KEY}-static:skins:${skinId ?? "none"}`);
                    if (skinsCached) {
                        return middleware.createResponse(skinsCached);
                    }

                    const skinsData = skinId ? skins(skinId) : getAllSkins();

                    await redis.set(
                        `${REDIS_KEY}-static:skins:${skinId ?? "none"}`,
                        JSON.stringify({
                            skins: skinsData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            skins: skinsData,
                        }),
                    );
                case "voices":
                    const voiceId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const voicesCached = await redis.get(`${REDIS_KEY}-static:voices:${voiceId ?? "none"}`);
                    if (voicesCached) {
                        return middleware.createResponse(voicesCached);
                    }

                    const voicesData = voiceId ? voices(voiceId) : getAllVoices();

                    await redis.set(
                        `${REDIS_KEY}-static:voices:${voiceId ?? "none"}`,
                        JSON.stringify({
                            voices: voicesData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(JSON.stringify({ voices: voicesData }));
                case "gacha":
                    const gachaMethod = body?.method ?? paths[2] ?? url.searchParams.get("method") ?? null;
                    if (!gachaMethod) {
                        const gachaCached = await redis.get(`${REDIS_KEY}-static:gacha`);
                        if (gachaCached) {
                            return middleware.createResponse(gachaCached);
                        }
                        return middleware.createResponse(JSON.stringify({ gacha: getAllGacha() }));
                    }

                    switch (gachaMethod.toLowerCase()) {
                        case "recruitment":
                            const recruitmentCached = await redis.get(`${REDIS_KEY}-static:gacha:recruitment`);
                            if (recruitmentCached) {
                                return middleware.createResponse(recruitmentCached);
                            }

                            const recruitmentData = getRecruitment();

                            await redis.set(`${REDIS_KEY}-static:gacha:recruitment`, JSON.stringify({ recruitment: recruitmentData }), "EX", env.REDIS_CACHE_TIME);

                            return middleware.createResponse(JSON.stringify({ recruitment: recruitmentData }));
                        case "calculate":
                            const recruitment = body?.recruitment ?? paths[3] ?? url.searchParams.get("recruitment") ?? null;

                            if (!recruitment) {
                                return middleware.createResponse(JSON.stringify({ error: "No recruitment provided." }), 400);
                            }

                            const rData = getRecruitment();

                            if (!rData) {
                                return middleware.createResponse(JSON.stringify({ error: "Internal server error. No recruitment data found." }), 400);
                            }

                            const recruitmentResult = calculateRecruitment(new Set(recruitment.split(",")), rData.RECRUIT_POOL, rData.TAG_MAP);

                            return middleware.createResponse(JSON.stringify({ recruitment: recruitmentResult }));
                        default:
                            return middleware.createResponse(JSON.stringify({ error: "Invalid method." }), 400);
                    }
                default:
                    return middleware.createResponse(JSON.stringify({ error: "Invalid type." }), 400);
            }
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
    path: "/static",
    handler,
    rateLimit: 50,
};

type Body = {
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook" | "skins" | "voices" | "recruitment";
    id?: string;
    method?: string;
    trust?: number;
    fields?: string[];
    recruitment?: string;
};

export default route;
