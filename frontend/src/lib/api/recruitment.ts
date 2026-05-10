import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { PROFESSION_LABELS } from "#/components/tools/recruitment/impl/constants";
import type { IGachaTag } from "#/components/tools/recruitment/impl/helpers";
import type { IRecruitableOperatorWithTags } from "#/components/tools/recruitment/impl/types";
import { deepCamelize } from "#/lib/api/operators";
import { backendFetch } from "#/lib/fetch";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";

interface IGachaDataResponse {
    gachaTags: IGachaTag[];
    recruitDetail: string;
}

export interface IRecruitmentData {
    tags: IGachaTag[];
    operators: IRecruitableOperatorWithTags[];
}

function parseRecruitableNames(recruitDetail: string): Set<string> {
    const names = new Set<string>();
    if (!recruitDetail) return names;

    const lines = recruitDetail.split("\n");
    for (const line of lines) {
        if (line.startsWith("<@rc.title>") || line.startsWith("<@rc.subtitle>") || line.startsWith("<@rc.em>") || line.startsWith("-----") || line.trim() === "" || /^★+$/.test(line.trim())) {
            continue;
        }

        // Strip markup tags before splitting on "/". Otherwise the "/" inside
        // a closing </> tag splits the tag across fragments, leaving a stray
        // "<" attached to every <@rc.eml>-wrapped operator name (e.g. "PhonoR-0<").
        const stripped = line.replace(/<[^>]*>/g, "");

        const lineNames = stripped
            .split("/")
            .map((n) => n.replace(/★+/g, "").trim())
            .filter((n) => n.length > 0);

        for (const name of lineNames) {
            names.add(name);
        }
    }

    return names;
}

function buildOperatorTagList(op: IOperatorListItem, rarity: number): string[] {
    const tags: string[] = [];
    if (op.position === "MELEE") tags.push("Melee");
    if (op.position === "RANGED") tags.push("Ranged");

    const profTag = PROFESSION_LABELS[op.profession];
    if (profTag) tags.push(profTag);

    if (rarity === 6) tags.push("Top Operator");
    if (rarity === 5) tags.push("Senior Operator");
    if (rarity === 1) tags.push("Robot");

    if (op.tagList) tags.push(...op.tagList);

    return tags;
}

export const getRecruitmentDataFn = createServerFn({ method: "GET" }).handler(async (): Promise<IRecruitmentData> => {
    const [gachaRes, opsRes] = await Promise.all([backendFetch("/static/gacha"), backendFetch("/static/operators")]);
    if (!gachaRes.ok) throw new Error(`Failed to load gacha data: ${gachaRes.status}`);
    if (!opsRes.ok) throw new Error(`Failed to load operators: ${opsRes.status}`);

    const gacha = (await gachaRes.json()) as IGachaDataResponse;
    const opsRaw = (await opsRes.json()) as IOperatorsStaticMap;
    const operatorsMap = deepCamelize(opsRaw);
    const allOperators = Object.values(operatorsMap) as IOperatorListItem[];

    const tags = gacha.gachaTags ?? [];
    const recruitableNames = parseRecruitableNames(gacha.recruitDetail ?? "");

    const operators: IRecruitableOperatorWithTags[] = [];
    for (const op of allOperators) {
        if (!op.id || !recruitableNames.has(op.name)) continue;
        const rarity = rarityToNumber(op.rarity);
        operators.push({
            id: op.id,
            name: op.name,
            rarity: `TIER_${rarity}`,
            profession: op.profession,
            position: op.position,
            tagList: buildOperatorTagList(op, rarity),
        });
    }

    return { tags, operators };
});

export function recruitmentDataQueryOptions() {
    return queryOptions({
        queryKey: ["recruitment", "data"],
        queryFn: () => getRecruitmentDataFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
