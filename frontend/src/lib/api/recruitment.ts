import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { toRecruitableOperator } from "#/components/tools/recruitment/impl/derive";
import type { IGachaTag } from "#/components/tools/recruitment/impl/helpers";
import type { IRecruitableOperator } from "#/components/tools/recruitment/impl/types";
import { deepCamelize } from "#/lib/api/operators";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

interface IGachaDataResponse {
    gachaTags: IGachaTag[];
    recruitDetail: string;
}

export interface IRecruitmentData {
    tags: IGachaTag[];
    operators: IRecruitableOperator[];
}

function parseRecruitableNames(recruitDetail: string): Set<string> {
    const names = new Set<string>();
    if (!recruitDetail) return names;

    const lines = recruitDetail.split("\n");
    for (const line of lines) {
        if (line.startsWith("<@rc.title>") || line.startsWith("<@rc.subtitle>") || line.startsWith("<@rc.em>") || line.startsWith("-----") || line.trim() === "" || /^★+$/.test(line.trim())) {
            continue;
        }

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

export const getRecruitmentDataFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }): Promise<IRecruitmentData> => {
        const [gachaRes, opsRes] = await Promise.all([backendFetch(gamedataPath(server, "/static/gacha")), backendFetch(gamedataPath(server, "/static/operators"))]);
        if (!gachaRes.ok) throw new Error(`Failed to load gacha data: ${gachaRes.status}`);
        if (!opsRes.ok) throw new Error(`Failed to load operators: ${opsRes.status}`);

        const gacha = (await gachaRes.json()) as IGachaDataResponse;
        const opsRaw = (await opsRes.json()) as IOperatorsStaticMap;
        const operatorsMap = deepCamelize(opsRaw);
        const allOperators = Object.values(operatorsMap) as IOperatorListItem[];

        const tags = gacha.gachaTags ?? [];
        const recruitableNames = parseRecruitableNames(gacha.recruitDetail ?? "");

        const operators: IRecruitableOperator[] = [];
        for (const op of allOperators) {
            if (!op.id || !recruitableNames.has(op.name)) continue;
            operators.push(toRecruitableOperator(op.id, op));
        }

        return { tags, operators };
    });

export function recruitmentDataQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["recruitment", "data", ...gamedataKey(server)],
        queryFn: () => getRecruitmentDataFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
