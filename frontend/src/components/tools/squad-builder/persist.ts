/**
 * What the Squad Builder page keeps in localStorage: the working copy with its
 * slot and saved baseline, the starting DP and where it came from, and the
 * compact toggle. Read once after mount, written as they change.
 */

import { readStoredValue, writeStoredValue } from "#/hooks/use-local-storage-state";
import { SLOT_LIMIT } from "./constants";
import { emptyDraft } from "./model";
import { DEFAULT_START_DP, parseStoredDraft, type StartDpFrom } from "./state";

export const DRAFT_KEY = "squad-builder:draft-v1";
export const START_DP_KEY = "squad-builder:start-dp-v1";
export const COMPACT_KEY = "squad-builder:compact-v1";

interface IStoredDraft {
    draft: unknown;
    slot: unknown;
    savedBaseline: unknown;
}

export function safeWrite(key: string, value: unknown): void {
    try {
        writeStoredValue(key, value);
    } catch {
        // Storage full or blocked: the page still works, it just will not remember.
    }
}

export function readHydration() {
    const stored = readStoredValue<IStoredDraft | null>(DRAFT_KEY, null);
    const dp = readStoredValue<{ startDp?: unknown; from?: unknown } | null>(START_DP_KEY, null);
    const compact = readStoredValue<unknown>(COMPACT_KEY, false);
    const slot = typeof stored?.slot === "number" && stored.slot >= 1 && stored.slot <= SLOT_LIMIT ? stored.slot : null;
    const from = dp?.from as StartDpFrom | undefined;
    return {
        draft: parseStoredDraft(stored?.draft) ?? emptyDraft(),
        slot,
        savedBaseline: slot !== null ? parseStoredDraft(stored?.savedBaseline) : null,
        startDp: typeof dp?.startDp === "number" && Number.isFinite(dp.startDp) && dp.startDp >= 0 ? dp.startDp : DEFAULT_START_DP,
        startDpFrom: from && from.kind === "stage" && typeof from.code === "string" ? from : ({ kind: "page" } as const),
        compact: compact === true,
    };
}
