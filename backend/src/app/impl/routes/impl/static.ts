import { calculateTrust, getMaterial, modules } from "../../../../lib/impl/local/impl/gamedata";
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
                    const materials = materialId ? await getMaterial(materialId) : await getAllMaterials();
                    return middleware.createResponse(
                        JSON.stringify({
                            materials,
                        }),
                    );
                case "modules":
                    const method = body?.method ?? paths[2] ?? url.searchParams.get("method") ?? null;
                    if (!method) {
                        const id = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;
                        const modulesData = id ? await modules.get(id) : await getAllModules();
                        return middleware.createResponse(
                            JSON.stringify({
                                modules: modulesData,
                            }),
                        );
                    } else {
                        switch (method.toLowerCase()) {
                            case "charid":
                                const charId = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;
                                const modulesData = await modules.getByCharId(charId);
                                return middleware.createResponse(
                                    JSON.stringify({
                                        modules: modulesData,
                                    }),
                                );
                            case "details":
                                const id = body?.id ?? paths[3] ?? url.searchParams.get("id") ?? null;
                                const details = await modules.getModuleDetails(id);
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
                    const operatorsData = operatorId ? await operators(operatorId) : await getAllOperators();
                    return middleware.createResponse(
                        JSON.stringify({
                            operators: operatorsData,
                        }),
                    );
                case "ranges":
                    const rangeId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;
                    const range = rangeId ? await ranges(rangeId) : await getAllRanges();
                    return middleware.createResponse(
                        JSON.stringify({
                            range,
                        }),
                    );
                case "skills":
                    const skillId = body?.id ?? paths[2] ?? url.searchParams.get("id") ?? null;
                    const skillsData = skillId ? await skills(skillId) : await getAllSkills();
                    return middleware.createResponse(
                        JSON.stringify({
                            skills: skillsData,
                        }),
                    );
                case "trust":
                    const trust = body?.trust ?? paths[2] ?? url.searchParams.get("trust") ?? null;
                    const trustData = trust ? await calculateTrust(Number(trust)) : null;
                    return middleware.createResponse(
                        JSON.stringify({
                            trust: trustData,
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
    rateLimit: 20,
};

type Body = {
    type: "materials" | "modules" | "operators" | "ranges" | "skills" | "trust";
    id?: string;
    method?: string;
    trust?: number;
};

export default route;
