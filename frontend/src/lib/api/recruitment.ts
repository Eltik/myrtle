import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { toRecruitableOperator } from "#/components/tools/recruitment/impl/derive";
import type { IGachaTag } from "#/components/tools/recruitment/impl/helpers";
import type { IRecruitableOperator } from "#/components/tools/recruitment/impl/types";
import { backendFetch } from "#/lib/fetch";
import type { RecruitmentData } from "#/types/generated/RecruitmentData";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

export interface IRecruitmentData {
    tags: IGachaTag[];
    operators: IRecruitableOperator[];
}

/**
 * The backend parses `recruitDetail` and serves only the recruitable operators,
 * with only the fields the calculator reads, from cache, so an SSR call never
 * pulls `/static/gacha` and the whole `/static/operators` table.
 */
export const getRecruitmentDataFn = createServerFn({ method: "GET" })
    .inputValidator((server: string | undefined) => server)
    .handler(async ({ data: server }): Promise<IRecruitmentData> => {
        const res = await backendFetch(gamedataPath(server, "/operators/recruitment"));
        if (!res.ok) throw new Error(`Failed to load recruitment data: ${res.status}`);
        const data = (await res.json()) as RecruitmentData;
        const tagNames = new Map(data.tags.map((tag) => [tag.tagId, tag.tagName] as const));
        return { tags: data.tags, operators: data.operators.map((op) => toRecruitableOperator(op, tagNames)) };
    });

export function recruitmentDataQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["recruitment", "data", ...gamedataKey(server)],
        queryFn: () => getRecruitmentDataFn({ data: resolveGamedataServer(server) }),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
