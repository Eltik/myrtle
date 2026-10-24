import { useQuery } from "@tanstack/react-query";
import { useShareHydration } from "#/components/tools/shared/ShareLink";
import { usePersistedReducer } from "#/components/tools/shared/usePersistedReducer";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { mergeWithDefaults } from "./persist";
import { decodeDpsShare, resolveDpsShare } from "./share";
import { dpsReducer, INITIAL_STATE } from "./state";
import type { IDpsState } from "./types";

const STORAGE_KEY = "dps-calculator-state-v1";

/** `shareParam` is the `?s=` link read at mount; it outranks the stored state. */
export function useDpsState(shareParam?: string) {
    const { data: operators, isError } = useQuery(dpsOperatorsQueryOptions());
    const external = useShareHydration(shareParam, decodeDpsShare, resolveDpsShare, operators, isError);
    return usePersistedReducer<IDpsState, Parameters<typeof dpsReducer>[1]>(STORAGE_KEY, dpsReducer, INITIAL_STATE, mergeWithDefaults, (state) => ({ type: "HYDRATE", state }), external);
}
