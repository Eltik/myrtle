import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { OperatorCnChanges } from "#/types/generated/OperatorCnChanges";
import type { IAttributeKeyFrame, IEnrichedSkill, IOperatorListItem, IOperatorModule } from "#/types/operators";

/** A blackboard row as skills, talents, traits and modules all carry it. */
type BlackboardRow = { key: string; value: number };

export type ChangeKind = OperatorChange["kind"];

/** Every change kind, in the order a summary lists them. */
export const CHANGE_KINDS: readonly ChangeKind[] = ["module_added", "module_changed", "skill_added", "skill_values_changed", "talents_changed", "trait_changed", "stats_changed", "range_changed", "potentials_changed", "trust_bonus_changed"];

/** Operator id -> its changes, for the list's badge and filter. */
export function cnChangesById(rows: readonly OperatorCnChanges[] | undefined): ReadonlyMap<string, OperatorChange[]> {
    const map = new Map<string, OperatorChange[]>();
    for (const row of rows ?? []) if (row.changes.length > 0) map.set(row.id, row.changes);
    return map;
}

/** The distinct kinds in `changes` with how often each occurs, in {@link CHANGE_KINDS} order. */
export function summarizeChanges(changes: readonly OperatorChange[]): { kind: ChangeKind; count: number }[] {
    const counts = new Map<ChangeKind, number>();
    for (const c of changes) counts.set(c.kind, (counts.get(c.kind) ?? 0) + 1);
    return CHANGE_KINDS.filter((k) => counts.has(k)).map((kind) => ({ kind, count: counts.get(kind) ?? 0 }));
}

/** One number that differs between the two servers; `null` where a side has no such entry. */
export interface IValueDiff {
    key: string;
    en: number | null;
    cn: number | null;
}

/** Same tolerance as the backend diff: the tables carry `f32` values widened to `f64`. */
function sameNumber(a: number, b: number): boolean {
    return Math.abs(a - b) <= 1e-6 * Math.max(1, Math.abs(a), Math.abs(b));
}

/**
 * The keys whose values differ between two blackboards, compared by lowercased key so a
 * reordered or differently cased blackboard reads as equal. Sorted by key.
 */
export function blackboardDiff(en: readonly BlackboardRow[] | null | undefined, cn: readonly BlackboardRow[] | null | undefined): IValueDiff[] {
    const toMap = (rows: readonly BlackboardRow[] | null | undefined) => new Map((rows ?? []).map((b) => [b.key.toLowerCase(), b.value] as const));
    const a = toMap(en);
    const b = toMap(cn);
    const keys = [...new Set([...a.keys(), ...b.keys()])].sort();
    return keys.flatMap((key) => {
        const x = a.get(key) ?? null;
        const y = b.get(key) ?? null;
        if (x !== null && y !== null && sameNumber(x, y)) return [];
        return [{ key, en: x, cn: y }];
    });
}

/** The numeric fields of two records that differ, by field name. */
function recordDiff(en: Record<string, unknown> | null | undefined, cn: Record<string, unknown> | null | undefined): IValueDiff[] {
    const keys = [...new Set([...Object.keys(en ?? {}), ...Object.keys(cn ?? {})])];
    return keys.flatMap((key) => {
        const x = en?.[key];
        const y = cn?.[key];
        const xn = typeof x === "number" ? x : null;
        const yn = typeof y === "number" ? y : null;
        if (xn === null && yn === null) return [];
        if (xn !== null && yn !== null && sameNumber(xn, yn)) return [];
        return [{ key, en: xn, cn: yn }];
    });
}

/** One skill level's differing numbers; `level` is 1-based (8 to 10 are the masteries). */
export interface ISkillLevelDiff {
    level: number;
    values: IValueDiff[];
}

/** Per level, the SP data, duration and blackboard numbers that differ between two copies of a skill. */
export function skillLevelDiffs(en: IEnrichedSkill | undefined, cn: IEnrichedSkill | undefined): ISkillLevelDiff[] {
    const a = en?.static?.levels ?? [];
    const b = cn?.static?.levels ?? [];
    const out: ISkillLevelDiff[] = [];
    for (let i = 0; i < Math.max(a.length, b.length); i++) {
        const x = a[i];
        const y = b[i];
        const values = [...recordDiff(x?.spData as Record<string, unknown> | undefined, y?.spData as Record<string, unknown> | undefined), ...recordDiff({ duration: x?.duration }, { duration: y?.duration }), ...blackboardDiff(x?.blackboard, y?.blackboard)];
        if (values.length > 0) out.push({ level: i + 1, values });
    }
    return out;
}

/** One promotion phase's differing stats at its max level (the last key frame). `phase` is 0-based. */
export interface IPhaseStatDiff {
    phase: number;
    level: number;
    values: IValueDiff[];
}

function lastFrame(frames: readonly IAttributeKeyFrame[] | undefined): IAttributeKeyFrame | undefined {
    return frames?.[frames.length - 1];
}

/** Per promotion phase, the max-level attributes that differ. */
export function phaseStatDiffs(en: IOperatorListItem, cn: IOperatorListItem): IPhaseStatDiff[] {
    const out: IPhaseStatDiff[] = [];
    for (let i = 0; i < Math.max(en.phases.length, cn.phases.length); i++) {
        const x = lastFrame(en.phases[i]?.attributesKeyFrames);
        const y = lastFrame(cn.phases[i]?.attributesKeyFrames);
        const values = recordDiff(x?.data as Record<string, unknown> | undefined, y?.data as Record<string, unknown> | undefined);
        if (values.length > 0) out.push({ phase: i, level: y?.level ?? x?.level ?? 0, values });
    }
    return out;
}

/** The trust bonus at full trust (the last favor key frame) where it differs. */
export function trustDiffs(en: IOperatorListItem, cn: IOperatorListItem): IValueDiff[] {
    return recordDiff(lastFrame(en.favorKeyFrames)?.data as Record<string, unknown> | undefined, lastFrame(cn.favorKeyFrames)?.data as Record<string, unknown> | undefined);
}

/** One talent or trait candidate's differing blackboard values, by talent and candidate position. */
export interface ICandidateDiff {
    talent: number;
    candidate: number;
    values: IValueDiff[];
}

/** Per talent candidate, the blackboard values that differ. */
export function talentDiffs(en: IOperatorListItem, cn: IOperatorListItem): ICandidateDiff[] {
    const out: ICandidateDiff[] = [];
    const a = en.talents ?? [];
    const b = cn.talents ?? [];
    for (let t = 0; t < Math.max(a.length, b.length); t++) {
        const x = a[t]?.candidates ?? [];
        const y = b[t]?.candidates ?? [];
        for (let c = 0; c < Math.max(x.length, y.length); c++) {
            const values = blackboardDiff(x[c]?.blackboard, y[c]?.blackboard);
            if (values.length > 0) out.push({ talent: t, candidate: c, values });
        }
    }
    return out;
}

/** Per trait candidate, the blackboard values that differ. */
export function traitDiffs(en: IOperatorListItem, cn: IOperatorListItem): IValueDiff[][] {
    const x = en.trait?.candidates ?? [];
    const y = cn.trait?.candidates ?? [];
    const out: IValueDiff[][] = [];
    for (let c = 0; c < Math.max(x.length, y.length); c++) {
        const values = blackboardDiff(x[c]?.blackboard, y[c]?.blackboard);
        if (values.length > 0) out.push(values);
    }
    return out;
}

/** Per module stage, the attribute bonuses that differ between two copies of a module. */
export function moduleStageDiffs(en: IOperatorModule | undefined, cn: IOperatorModule | undefined): { stage: number; values: IValueDiff[] }[] {
    const a = en?.data?.phases ?? [];
    const b = cn?.data?.phases ?? [];
    const out: { stage: number; values: IValueDiff[] }[] = [];
    for (let i = 0; i < Math.max(a.length, b.length); i++) {
        const values = blackboardDiff(a[i]?.attributeBlackboard, b[i]?.attributeBlackboard);
        if (values.length > 0) out.push({ stage: a[i]?.equipLevel ?? b[i]?.equipLevel ?? i + 1, values });
    }
    return out;
}
