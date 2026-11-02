import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { OperatorCnChanges } from "#/types/generated/OperatorCnChanges";
import type { IOperatorListItem } from "#/types/operators";

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

/** An operator with CN's additions merged in, and which of its modules and skills are CN's. */
export interface ICnMerged {
    operator: IOperatorListItem;
    moduleIds: ReadonlySet<string>;
    skillIds: ReadonlySet<string>;
}

/**
 * `en` with the modules and skills `changes` reports as added on CN appended from CN's record,
 * so the page's own module and skill pickers offer them after this server's. Everything else
 * stays `en`'s; with nothing added, `operator` is `en` itself.
 */
export function withCnAdditions(en: IOperatorListItem, cn: IOperatorListItem, changes: readonly OperatorChange[]): ICnMerged {
    const added = new Set(changes.flatMap((c) => (c.kind === "module_added" ? [c.moduleId] : c.kind === "skill_added" ? [c.skillId] : [])));
    const modules = cn.modules.filter((m) => added.has(m.uniEquipId) && !en.modules.some((x) => x.uniEquipId === m.uniEquipId));
    const skills = cn.skills.filter((s) => added.has(s.skillId) && !en.skills.some((x) => x.skillId === s.skillId));
    return {
        operator: modules.length === 0 && skills.length === 0 ? en : { ...en, modules: [...en.modules, ...modules], skills: [...en.skills, ...skills] },
        moduleIds: new Set(modules.map((m) => m.uniEquipId)),
        skillIds: new Set(skills.map((s) => s.skillId)),
    };
}
