import { CHART_PALETTE, getInstanceColor } from "./constants";
import { MAX_SKILL_RANK } from "./skill";
import type { IBuildConfig, ICalcBuffs, ICalcConditionals, IConditionalInfo, IInstance, IOperatorListEntry } from "./types";

export function newInstanceId(): string {
    if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
        return crypto.randomUUID();
    }
    return `i-${Math.random().toString(36).slice(2)}-${Date.now().toString(36)}`;
}

/** First palette color not in use; wraps the palette once all are taken. */
export function pickAvailableColor(used: ReadonlySet<string>, instanceCount: number): string {
    for (const color of CHART_PALETTE) {
        if (!used.has(color)) return color;
    }
    return getInstanceColor(instanceCount);
}

/** Resolve an operator's recommended conditional defaults. */
export function defaultConditionals(op: IOperatorListEntry): ICalcConditionals {
    const conditionals: ICalcConditionals = {};
    for (const cond of op.conditionals) {
        const key = conditionalKey(cond);
        if (key && conditionals[key] === undefined) conditionals[key] = cond.default;
    }
    return conditionals;
}

/** Default build config for a freshly added operator (max rank = M3). */
export function defaultBuildConfig(op: IOperatorListEntry): IBuildConfig {
    return {
        potential: 1,
        trust: 100,
        skillIndex: op.defaultSkill,
        skillRank: MAX_SKILL_RANK,
        moduleIndex: op.defaultModule,
        moduleLevel: 3,
        buffs: {},
        conditionals: defaultConditionals(op),
        allCond: true,
    };
}

export function createInstance(op: IOperatorListEntry, color: string): IInstance {
    return { uid: newInstanceId(), op, color, visible: true, collapsed: false, config: defaultBuildConfig(op) };
}

/** Strip zero/undefined numeric entries; returns undefined when nothing is set. */
export function stripEmptyNumbers(obj: Record<string, number | undefined>): Record<string, number> | undefined {
    const out: Record<string, number> = {};
    let any = false;
    for (const [k, v] of Object.entries(obj)) {
        if (typeof v === "number" && v !== 0) {
            out[k] = v;
            any = true;
        }
    }
    return any ? out : undefined;
}

/** Map a backend conditional type to its request flag key. */
export function conditionalKey(cond: IConditionalInfo): keyof ICalcConditionals | null {
    switch (cond.conditionalType) {
        case "trait":
            return "traitDamage";
        case "talent":
            return "talentDamage";
        case "talent2":
            return "talent2Damage";
        case "skill":
            return "skillDamage";
        case "module":
            return "moduleDamage";
        default:
            return null;
    }
}

/** Actions that mutate the `instances` array; apply them via {@link reduceInstances}. */
export type InstanceAction =
    | { type: "ADD_INSTANCE"; op: IOperatorListEntry }
    | { type: "DUPLICATE_INSTANCE"; uid: string }
    | { type: "REMOVE_INSTANCE"; uid: string }
    | { type: "UPDATE_CONFIG"; uid: string; patch: Partial<IBuildConfig> }
    | { type: "UPDATE_BUFFS"; uid: string; patch: Partial<ICalcBuffs> }
    | { type: "TOGGLE_CONDITIONAL"; uid: string; key: keyof ICalcConditionals; value: boolean }
    | { type: "TOGGLE_VISIBILITY"; uid: string }
    | { type: "TOGGLE_COLLAPSED"; uid: string }
    | { type: "SET_ALL_COLLAPSED"; collapsed: boolean }
    | { type: "REORDER_INSTANCE"; uid: string; direction: "up" | "down" }
    | { type: "RESET_INSTANCES" }
    | { type: "REFRESH_OPS"; freshOps: Map<string, IOperatorListEntry> };

/** Returns the (possibly unchanged) instances array for an instance action. */
export function reduceInstances(instances: IInstance[], action: InstanceAction): IInstance[] {
    switch (action.type) {
        case "ADD_INSTANCE": {
            const used = new Set(instances.map((i) => i.color));
            return [...instances, createInstance(action.op, pickAvailableColor(used, instances.length))];
        }
        case "DUPLICATE_INSTANCE": {
            const src = instances.find((i) => i.uid === action.uid);
            if (!src) return instances;
            const used = new Set(instances.map((i) => i.color));
            const clone: IInstance = {
                ...src,
                uid: newInstanceId(),
                color: pickAvailableColor(used, instances.length),
                config: { ...src.config, buffs: { ...src.config.buffs }, conditionals: { ...src.config.conditionals } },
            };
            return [...instances, clone];
        }
        case "REMOVE_INSTANCE":
            return instances.filter((i) => i.uid !== action.uid);
        case "UPDATE_CONFIG":
            return instances.map((i) => (i.uid === action.uid ? { ...i, config: { ...i.config, ...action.patch } } : i));
        case "UPDATE_BUFFS":
            return instances.map((i) => (i.uid === action.uid ? { ...i, config: { ...i.config, buffs: { ...i.config.buffs, ...action.patch } } } : i));
        case "TOGGLE_CONDITIONAL":
            return instances.map((i) => (i.uid === action.uid ? { ...i, config: { ...i.config, conditionals: { ...i.config.conditionals, [action.key]: action.value } } } : i));
        case "TOGGLE_VISIBILITY":
            return instances.map((i) => (i.uid === action.uid ? { ...i, visible: !i.visible } : i));
        case "TOGGLE_COLLAPSED":
            return instances.map((i) => (i.uid === action.uid ? { ...i, collapsed: !i.collapsed } : i));
        case "SET_ALL_COLLAPSED":
            return instances.map((i) => (i.collapsed === action.collapsed ? i : { ...i, collapsed: action.collapsed }));
        case "REORDER_INSTANCE": {
            const idx = instances.findIndex((i) => i.uid === action.uid);
            if (idx === -1) return instances;
            const target = action.direction === "up" ? idx - 1 : idx + 1;
            if (target < 0 || target >= instances.length) return instances;
            const next = [...instances];
            const [moved] = next.splice(idx, 1);
            next.splice(target, 0, moved);
            return next;
        }
        case "RESET_INSTANCES":
            return [];
        case "REFRESH_OPS": {
            let dirty = false;
            const next = instances.map((inst) => {
                const fresh = action.freshOps.get(inst.op.id);
                if (!fresh || fresh === inst.op) return inst;
                dirty = true;
                return { ...inst, op: fresh };
            });
            return dirty ? next : instances;
        }
    }
}

/**
 * Normalise a persisted instance to the current config shape (deriving
 * `skillRank` from an older `masteryLevel`, defaulting any missing fields).
 * Returns null when the value is unusable.
 */
export function migrateInstance(raw: unknown): IInstance | null {
    if (!raw || typeof raw !== "object") return null;
    const inst = raw as IInstance & { config?: IBuildConfig & { masteryLevel?: number } };
    if (!inst.op || !inst.config) return null;
    const cfg = inst.config;
    if (cfg.skillRank === undefined && cfg.masteryLevel !== undefined) {
        // Old encoding: masteryLevel 0=L7, 1/2/3=M1/M2/M3 → ranks 7/8/9/10.
        cfg.skillRank = cfg.masteryLevel <= 0 ? 7 : 7 + cfg.masteryLevel;
    }
    if (cfg.skillRank === undefined) cfg.skillRank = MAX_SKILL_RANK;
    if (!cfg.buffs) cfg.buffs = {};
    return inst;
}
