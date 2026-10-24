import { useQuery } from "@tanstack/react-query";
import { useCallback, useMemo } from "react";
import { dpsSupportersQueryOptions } from "#/lib/api/dps";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import type { SupporterEntry } from "#/types/generated/SupporterEntry";
import { teamBuffsGate } from "./teamBuffsGate";
import type { ITeamBuffs } from "./types";
import { type ISupporterSource, supporterSources } from "./zones";

export interface ITeamBuffsContext {
    /** The switched-on supporters' contributions, values from game data at the team investment. */
    sources: ISupporterSource[];
    /** Every resolved entry at that investment; undefined until loaded. */
    entries: SupporterEntry[] | undefined;
    /**
     * Requests must wait: supporters are switched on and their values have not arrived, or a
     * class-limited one is on and the operator index (which says who it reaches) has not.
     */
    waiting: boolean;
    /**
     * The supporter values or the operator index failed to load while needed. Requests are
     * NOT sent then: computing without the switched-on supporters would show numbers that
     * silently leave them out.
     */
    error: Error | null;
    /** Game profession code per operator id, for supporters limited to a class; undefined until the index loads. */
    professionOf: (opId: string) => string | undefined;
}

/** Resolve the team panel's supporter ids into zone sources. The supporters list is fetched only once one is switched on. */
export function useTeamBuffs(team: ITeamBuffs): ITeamBuffsContext {
    const enabled = team.supporters.length > 0;
    const supporters = useQuery({ ...dpsSupportersQueryOptions(team.investment), enabled });
    const sources = useMemo(() => supporterSources(supporters.data?.entries, team.supporters), [supporters.data, team.supporters]);
    const scoped = sources.some((s) => s.professions.length > 0);
    const index = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled: scoped });
    const byId = useMemo(() => new Map((index.data ?? []).map((o) => [o.id, o.profession as string])), [index.data]);
    const professionOf = useCallback((opId: string) => byId.get(opId), [byId]);
    return { sources, entries: supporters.data?.entries, professionOf, ...teamBuffsGate({ enabled, supporters, scoped, index }) };
}
