import type { IOperatorListItem } from "#/types/operators";
import type { IVoices } from "#/types/voices";
import type { IOperatorStats, IOperatorView } from "./types";

function extractVoiceActors(charId: string, voices: IVoices | undefined): string[] {
    const entry = voices?.voiceLangDict[charId];
    if (!entry) return [];
    const names = new Set<string>();
    for (const lang of Object.values(entry.dict)) {
        for (const name of lang.cvName) names.add(name);
    }
    return [...names];
}

function extractStats(op: IOperatorListItem): IOperatorStats | null {
    const lastPhase = op.phases[op.phases.length - 1];
    const lastFrame = lastPhase?.attributesKeyFrames ? lastPhase?.attributesKeyFrames[lastPhase?.attributesKeyFrames.length - 1] : undefined;
    if (!lastFrame) return null;
    const a = lastFrame.data;
    return {
        hp: a.maxHp,
        atk: a.atk,
        def: a.def,
        res: a.magicResistance,
        cost: a.cost,
        block: a.blockCnt,
    };
}

export function enrichOperator(op: IOperatorListItem, voices: IVoices | undefined): IOperatorView {
    const info = op.profile?.basicInfo;
    return {
        ...op,
        gender: info?.gender ?? null,
        race: info?.race ?? null,
        placeOfBirth: info?.placeOfBirth ?? null,
        voiceActors: op.id ? extractVoiceActors(op.id, voices) : [],
        stats: extractStats(op),
    };
}

export function enrichOperators(ops: IOperatorListItem[], voices: IVoices | undefined): IOperatorView[] {
    return ops.map((op) => enrichOperator(op, voices));
}
