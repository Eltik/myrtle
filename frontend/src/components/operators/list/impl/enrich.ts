import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IOperatorOwnershipInfo, IOperatorView } from "./types";

export type NotedSet = ReadonlySet<string>;
export type OwnershipLookup = ReadonlyMap<string, IOperatorOwnershipInfo>;
export type CnChangesLookup = ReadonlyMap<string, OperatorChange[]>;

export function enrichOperator(op: IOperatorIndexEntry, notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined, cnChanges?: CnChangesLookup): IOperatorView {
    return {
        ...op,
        hasNotes: op.id ? (notedIds?.has(op.id) ?? false) : false,
        ownership: op.id ? (ownership?.get(op.id) ?? null) : null,
        cnChanges: op.id ? (cnChanges?.get(op.id) ?? null) : null,
    };
}

export function enrichOperators(ops: IOperatorIndexEntry[], notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined, cnChanges?: CnChangesLookup): IOperatorView[] {
    return ops.map((op) => enrichOperator(op, notedIds, ownership, cnChanges));
}
