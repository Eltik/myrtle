import { useQuery, useQueryClient } from "@tanstack/react-query";
import * as React from "react";
import { putReleasePlanFn, releasePlanQueryOptions } from "#/lib/api/release";
import type { PutReleasePlan } from "#/types/generated/PutReleasePlan";
import type { ReleasePlan } from "#/types/generated/ReleasePlan";
import { EMPTY_STATE, type IPlanState, loadState, saveState } from "./plan";

const SAVE_DEBOUNCE_MS = 800;

function cleanStages(raw: Record<string, Record<string, boolean | undefined> | undefined>): Record<string, Record<string, boolean>> {
    const stages: Record<string, Record<string, boolean>> = {};
    for (const [row, codes] of Object.entries(raw)) {
        const clean: Record<string, boolean> = {};
        for (const [code, on] of Object.entries(codes ?? {})) if (on !== undefined) clean[code] = on;
        stages[row] = clean;
    }
    return stages;
}

function fromServer(plan: ReleasePlan): IPlanState {
    const picks: Record<string, true> = {};
    for (const id of plan.picks) picks[id] = true;
    return { initial: plan.initial, initialManual: plan.initialManual, picks, stages: cleanStages(plan.stages) };
}

function toServer(state: IPlanState): PutReleasePlan {
    return { initial: state.initial, initialManual: state.initialManual, picks: Object.keys(state.picks), stages: cleanStages(state.stages) };
}

export interface ISync {
    saving: boolean;
    savedAt: number | null;
}

export function useStoredState(uid: string | null): [IPlanState, React.Dispatch<React.SetStateAction<IPlanState>>, ISync] {
    const queryClient = useQueryClient();
    const server = useQuery(releasePlanQueryOptions(uid));
    const [state, setState] = React.useState<IPlanState>(EMPTY_STATE);
    const source = React.useRef<"none" | "local" | "server">("none");
    const dirty = React.useRef(false);
    const [saving, setSaving] = React.useState(false);
    const [savedAt, setSavedAt] = React.useState<number | null>(null);

    React.useEffect(() => {
        if (uid || source.current === "local") return;
        setState(loadState());
        source.current = "local";
    }, [uid]);

    React.useEffect(() => {
        if (!uid || !server.isSuccess || source.current === "server") return;
        source.current = "server";
        if (server.data) {
            setState(fromServer(server.data));
            setSavedAt(server.data.updatedAt);
        } else {
            const local = loadState();
            setState(local);
            dirty.current = local !== EMPTY_STATE;
        }
    }, [uid, server.isSuccess, server.data]);

    const first = React.useRef(true);
    React.useEffect(() => {
        if (first.current) {
            first.current = false;
            return;
        }
        if (source.current === "none") return;
        saveState(state);
        if (source.current !== "server") return;
        dirty.current = true;
        const handle = setTimeout(async () => {
            if (!dirty.current) return;
            dirty.current = false;
            setSaving(true);
            try {
                const saved = await putReleasePlanFn({ data: toServer(state) });
                setSavedAt(saved.updatedAt);
                queryClient.setQueryData(releasePlanQueryOptions(uid).queryKey, saved);
            } catch {
                dirty.current = true;
            } finally {
                setSaving(false);
            }
        }, SAVE_DEBOUNCE_MS);
        return () => clearTimeout(handle);
    }, [state, uid, queryClient]);

    return [state, setState, { saving, savedAt }];
}
