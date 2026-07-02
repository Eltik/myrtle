import type { IOperatorIndexEntry } from "#/types/operators";
import type { IVoices } from "#/types/voices";
import type { IOperatorOwnershipInfo, IOperatorView } from "./types";

export type NotedSet = ReadonlySet<string>;
export type OwnershipLookup = ReadonlyMap<string, IOperatorOwnershipInfo>;

function extractVoiceActors(charId: string, voices: IVoices | undefined): string[] {
    const entry = voices?.voiceLangDict[charId];
    if (!entry) return [];
    const names = new Set<string>();
    for (const lang of Object.values(entry.dict)) {
        for (const name of lang.cvName) names.add(name);
    }
    return [...names];
}

export function enrichOperator(op: IOperatorIndexEntry, voices: IVoices | undefined, notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined): IOperatorView {
    return {
        ...op,
        voiceActors: op.id ? extractVoiceActors(op.id, voices) : [],
        hasNotes: op.id ? (notedIds?.has(op.id) ?? false) : false,
        ownership: op.id ? (ownership?.get(op.id) ?? null) : null,
    };
}

export function enrichOperators(ops: IOperatorIndexEntry[], voices: IVoices | undefined, notedIds: NotedSet | undefined, ownership: OwnershipLookup | undefined): IOperatorView[] {
    return ops.map((op) => enrichOperator(op, voices, notedIds, ownership));
}
