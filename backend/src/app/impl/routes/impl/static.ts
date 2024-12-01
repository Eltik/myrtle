import { redis } from "../../..";
import { env } from "../../../../env";
import { calculateTrust, getHandbook, getMaterial, modules } from "../../../../lib/impl/local/impl/gamedata";
import { getAll as getAllMaterials } from "../../../../lib/impl/local/impl/gamedata/impl/materials";
import { getAll as getAllModules } from "../../../../lib/impl/local/impl/gamedata/impl/modules";
import operators, { getAll as getAllOperators } from "../../../../lib/impl/local/impl/gamedata/impl/operators";
import ranges, { getAll as getAllRanges } from "../../../../lib/impl/local/impl/gamedata/impl/ranges";
import skills, { getAll as getAllSkills } from "../../../../lib/impl/local/impl/gamedata/impl/skills";
import middleware from "../../middleware";

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

                    const materialsCached = await redis.get(`static:materials:${materialId ?? "none"}`);
                    if (materialsCached) {
                        return middleware.createResponse(materialsCached);
                    }

                    const materials = materialId ? getMaterial(materialId) : getAllMaterials();

                    await redis.set(
                        `static:materials:${materialId ?? "none"}`,
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

                        const modulesIdCached = await redis.get(`static:modules:${id ?? "none"}`);
                        if (modulesIdCached) {
                            return middleware.createResponse(modulesIdCached);
                        }

                        const modulesData = id ? modules.get(id) : await getAllModules();

                        await redis.set(
                            `static:modules:${id ?? "none"}`,
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

                                const modulesCharIdCached = await redis.get(`static:modules:charid:${charId ?? "none"}`);
                                if (modulesCharIdCached) {
                                    return middleware.createResponse(modulesCharIdCached);
                                }

                                const modulesData = modules.getByCharId(charId);

                                await redis.set(
                                    `static:modules:charid:${charId ?? "none"}`,
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

                                const modulesDetailsCached = await redis.get(`static:modules:details:${id ?? "none"}`);
                                if (modulesDetailsCached) {
                                    return middleware.createResponse(modulesDetailsCached);
                                }

                                const details = modules.getModuleDetails(id);

                                await redis.set(
                                    `static:modules:details:${id ?? "none"}`,
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

                    const operatorsCached = await redis.get(`static:operators:${operatorId ?? "none"}`);
                    if (operatorsCached) {
                        return middleware.createResponse(operatorsCached);
                    }

                    const operatorsData = operatorId ? operators(operatorId) : getAllOperators();

                    await redis.set(
                        `static:operators:${operatorId ?? "none"}`,
                        JSON.stringify({
                            operators: operatorsData,
                        }),
                        "EX",
                        env.REDIS_CACHE_TIME,
                    );

                    return middleware.createResponse(
                        JSON.stringify({
                            operators: operatorsData,
                        }),
                    );
                case "ranges":
                    const rangeId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;

                    const rangesCached = await redis.get(`static:ranges:${rangeId ?? "none"}`);
                    if (rangesCached) {
                        return middleware.createResponse(rangesCached);
                    }

                    const range = rangeId ? ranges(rangeId) : getAllRanges();

                    await redis.set(
                        `static:ranges:${rangeId ?? "none"}`,
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

                    const skillsCached = await redis.get(`static:skills:${skillId ?? "none"}`);
                    if (skillsCached) {
                        return middleware.createResponse(skillsCached);
                    }

                    const skillsData = skillId ? skills(skillId) : getAllSkills();

                    await redis.set(
                        `static:skills:${skillId ?? "none"}`,
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

                    const trustCached = await redis.get(`static:trust:${trust ?? "none"}`);
                    if (trustCached) {
                        return middleware.createResponse(trustCached);
                    }

                    const trustData = trust ? calculateTrust(Number(trust)) : null;

                    await redis.set(
                        `static:trust:${trust ?? "none"}`,
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

                    const handbookCached = await redis.get(`static:handbook:${charId ?? "none"}`);
                    if (handbookCached) {
                        return middleware.createResponse(handbookCached);
                    }

                    const handbookData = charId ? getHandbook(charId) : null;

                    await redis.set(
                        `static:handbook:${charId ?? "none"}`,
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
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust" | "handbook";
    id?: string;
    method?: string;
    trust?: number;
};

export default route;
