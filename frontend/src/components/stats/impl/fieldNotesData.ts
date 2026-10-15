import type { IEnemy, IEnemyLevel } from "#/lib/api/enemies";

/**
 * The statuses both sides of the data share: an enemy carries `${status}Immune`, and an
 * operator skill names the status as a segment of a blackboard key.
 */
export const FIELD_NOTE_STATUSES = ["stun", "silence", "sleep", "frozen", "levitate"] as const;
export type FieldNoteStatus = (typeof FIELD_NOTE_STATUSES)[number];
export const DEFAULT_FIELD_NOTE_STATUS: FieldNoteStatus = "stun";

const IMMUNE_FLAG: Record<FieldNoteStatus, "stunImmune" | "silenceImmune" | "sleepImmune" | "frozenImmune" | "levitateImmune"> = {
    stun: "stunImmune",
    silence: "silenceImmune",
    sleep: "sleepImmune",
    frozen: "frozenImmune",
    levitate: "levitateImmune",
};

export interface IRatio {
    n: number;
    total: number;
}

/** `n / total` as a fraction, or null when there is nothing to divide by. */
export function share({ n, total }: IRatio): number | null {
    return total > 0 ? n / total : null;
}

/**
 * Whether a blackboard key names `status` as a whole segment. Mirrors the backend's
 * `key_has_token` (`backend/src/core/squad/coverage.rs`): the key splits on `@ . [ ]`, and a
 * segment matches when it IS the status or starts with `status_`. So `stun`,
 * `stun_duration`, `attack@sluggish` (for sluggish) and `sleach_t_1[enemy].stun` match, while
 * `stunned`, `max_stun` and `sluggish` do not match `stun`. Case-insensitive.
 */
export function keyHasStatus(key: string, status: string): boolean {
    const tok = status.toLowerCase();
    return key
        .toLowerCase()
        .split(/[@.[\]]/)
        .some((seg) => seg === tok || seg.startsWith(`${tok}_`));
}

export interface IEnemyImmunity extends IRatio {
    byLevel: Record<IEnemyLevel, IRatio>;
}

/**
 * Handbook enemies immune to `status`. Hidden enemies are left out, and so are enemies with
 * no level-0 attributes: a missing stat block is unknown, not "not immune".
 */
export function enemyImmunity(enemies: readonly IEnemy[], status: FieldNoteStatus): IEnemyImmunity {
    const flag = IMMUNE_FLAG[status];
    const byLevel: Record<IEnemyLevel, IRatio> = { BOSS: { n: 0, total: 0 }, ELITE: { n: 0, total: 0 }, NORMAL: { n: 0, total: 0 } };
    let n = 0;
    let total = 0;
    for (const enemy of enemies) {
        if (enemy.hideInHandbook) continue;
        const attrs = enemy.stats?.levels?.[0]?.attributes;
        if (!attrs) continue;
        const immune = attrs[flag] === true;
        total += 1;
        if (immune) n += 1;
        const bucket = byLevel[enemy.enemyLevel];
        if (bucket) {
            bucket.total += 1;
            if (immune) bucket.n += 1;
        }
    }
    return { n, total, byLevel };
}

/** The slice of an index row the facts read. */
export interface IFieldNoteOperator {
    id: string;
    name: string;
    rarity: number;
    isNotObtainable?: boolean;
}

/** One skill level as the facts read it: its rich-text description and its blackboard. */
export interface IFieldNoteSkillLevel {
    description?: string | null;
    blackboard?: ReadonlyArray<{ key: string; value: number }> | null;
}

/** The slice of a roster row the facts read: skills, each with its levels. */
export interface IFieldNoteRosterRow {
    id: string | null;
    skills?: ReadonlyArray<{ static?: { levels?: ReadonlyArray<IFieldNoteSkillLevel> | null } | null }> | null;
}

/**
 * The control statuses the game marks up as `<$ba.status>` terms in skill descriptions.
 * Silence is absent: its text is never tagged, so it is only ever read off the keys.
 */
const TAGGED_CONTROL: readonly string[] = ["stun", "sluggish", "sleep", "levitate", "cold", "frozen", "root", "binding", "fear", "palsy", "tremble", "attract", "liftoff"];

/**
 * Whether one skill level inflicts `status`. When the description tags control statuses, the
 * tags are the list; when it tags none, or `status` has no tag at all (silence), a nonzero
 * blackboard key naming it decides.
 *
 * Keys alone over-count: Mudrock's S3 `sleep` is her own invulnerable phase (the text tags
 * only Stun), and the `frozen` keys of Magallan, Glaucus and Aosta time a Bind (`$ba.root`).
 * Tags alone miss untagged prose: Typhon, Penance and Ulpianus stun in plain text, and no
 * silence is ever tagged. Measured on the EN roster 2026-10-13, operators matched, keys only
 * -> this rule: stun 54 -> 59, silence 9 -> 9, sleep 5 -> 9, frozen 3 -> 2, levitate 5 -> 6.
 * Known miss, a TRADE: Saileach's S2 stun is untagged while the same text tags Slow, the
 * exact shape of Mudrock's self-sleep, and nothing short of reading the prose separates them.
 */
export function levelInflicts(level: IFieldNoteSkillLevel, status: string): boolean {
    const description = level.description ?? "";
    const tagged = TAGGED_CONTROL.filter((tag) => description.includes(`<$ba.${tag}>`));
    if (tagged.length > 0 && TAGGED_CONTROL.includes(status)) return tagged.includes(status);
    return (level.blackboard ?? []).some((entry) => entry.value !== 0 && keyHasStatus(entry.key, status));
}

/**
 * The operator denominator: index rows a player can own. `isNotObtainable` rows (the
 * Integrated Strategies reserve operators and the like) are dropped, since nobody can own
 * them and they would always win "rarest".
 */
export function ownableOperators<T extends IFieldNoteOperator>(index: readonly T[]): T[] {
    return index.filter((op) => !op.isNotObtainable);
}

/** Whether any level of any skill inflicts `status`, by `levelInflicts`. */
export function rowInflicts(row: IFieldNoteRosterRow, status: string): boolean {
    return (row.skills ?? []).some((skill) => (skill.static?.levels ?? []).some((level) => levelInflicts(level, status)));
}

export interface IOperatorsInflicting extends IRatio {
    ids: string[];
}

/** Operators in `operators` with a skill that inflicts `status`; roster rows outside it are ignored. */
export function operatorsInflicting(operators: readonly IFieldNoteOperator[], roster: readonly IFieldNoteRosterRow[], status: string): IOperatorsInflicting {
    const rows = new Map(roster.map((row) => [row.id, row]));
    const ids: string[] = [];
    for (const op of operators) {
        const row = rows.get(op.id);
        if (row && rowInflicts(row, status)) ids.push(op.id);
    }
    return { n: ids.length, total: operators.length, ids };
}

export interface IOwnershipLike {
    totalUsers: number;
    counts: { [id: string]: { owners: number } | undefined };
}

export interface IOwnedOperator extends IRatio {
    id: string;
    name: string;
}

function pickByOwners(operators: readonly IFieldNoteOperator[], ownership: IOwnershipLike, better: (a: number, b: number) => boolean): IOwnedOperator | null {
    let best: IOwnedOperator | null = null;
    for (const op of operators) {
        const owners = ownership.counts[op.id]?.owners ?? 0;
        if (!best || better(owners, best.n) || (owners === best.n && op.id < best.id)) {
            best = { id: op.id, name: op.name, n: owners, total: ownership.totalUsers };
        }
    }
    return best;
}

/**
 * The 6★ the most doctors own; ties go to the lower id. Restricted to 6★ because across all
 * rarities the answer is a gifted starter: Amiya, Noir Corne, Rangers and Yato each sit at
 * 2580 of 2580 (2026-10-13), which says nothing.
 */
export function mostOwnedSixStar(operators: readonly IFieldNoteOperator[], ownership: IOwnershipLike): IOwnedOperator | null {
    return pickByOwners(
        operators.filter((op) => op.rarity === 6),
        ownership,
        (a, b) => a > b,
    );
}

/** The 6★ the fewest doctors own (a missing id is 0 owners); ties go to the lower id. */
export function rarestSixStar(operators: readonly IFieldNoteOperator[], ownership: IOwnershipLike): IOwnedOperator | null {
    return pickByOwners(
        operators.filter((op) => op.rarity === 6),
        ownership,
        (a, b) => a < b,
    );
}
