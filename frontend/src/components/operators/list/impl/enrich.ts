import type { IOperatorIndexEntry } from "#/types/operators";
import type { IOperatorOwnershipInfo, IOperatorView } from "./types";

export type NotedSet = ReadonlySet<string>;
export type OwnershipLookup = ReadonlyMap<string, IOperatorOwnershipInfo>;

export function enrichOperator(op: IOperatorIndexEntry, notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined): IOperatorView {
    return {
        ...op,
        hasNotes: op.id ? (notedIds?.has(op.id) ?? false) : false,
        ownership: op.id ? (ownership?.get(op.id) ?? null) : null,
    };
}

export function enrichOperators(ops: IOperatorIndexEntry[], notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined): IOperatorView[] {
    return ops.map((op) => enrichOperator(op, notedIds, ownership));
}
