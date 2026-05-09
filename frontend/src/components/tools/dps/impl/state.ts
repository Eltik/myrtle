import { buildDefaultDpsRequest, type IDpsCalculateConditionals, type IDpsCalculateRequest, type IDpsOperatorListEntry } from "#/lib/api/dps";
import { CHART_PALETTE, DEFAULT_SWEEP, getInstanceColor } from "./constants";
import type { DpsAction, IDpsInstance, IDpsState, IEnemyConfig } from "./types";

const DEFAULT_ENEMY: IEnemyConfig = {
    defense: 0,
    res: 0,
    targets: 1,
    spBoost: 0,
    shred: { def: 0, defFlat: 0, res: 0, resFlat: 0 },
};

export const INITIAL_STATE: IDpsState = {
    instances: [],
    enemy: DEFAULT_ENEMY,
    xAxis: "defense",
    yMetric: "skill_dps",
    sweep: { ...DEFAULT_SWEEP },
};

export function createInstance(op: IDpsOperatorListEntry, color: string): IDpsInstance {
    const defaults = buildDefaultDpsRequest(op);
    const conditionals: IDpsCalculateConditionals = { ...defaults.conditionals };
    const allCondLevels: Record<string, boolean> = {};
    for (const cond of op.conditionals) {
        switch (cond.conditionalType) {
            case "trait":
                if (conditionals.traitDamage === undefined) conditionals.traitDamage = cond.default;
                break;
            case "talent":
                if (conditionals.talentDamage === undefined) conditionals.talentDamage = cond.default;
                break;
            case "talent2":
                if (conditionals.talent2Damage === undefined) conditionals.talent2Damage = cond.default;
                break;
            case "skill":
                if (conditionals.skillDamage === undefined) conditionals.skillDamage = cond.default;
                break;
            case "module":
                if (conditionals.moduleDamage === undefined) conditionals.moduleDamage = cond.default;
                break;
        }
        allCondLevels[cond.conditionalType] = cond.default;
    }
    return {
        uid: cryptoRandomId(),
        op,
        color,
        visible: true,
        collapsed: false,
        config: {
            potential: 1,
            trust: 100,
            skillIndex: defaults.skillIndex ?? op.defaultSkill,
            masteryLevel: 3,
            moduleIndex: defaults.moduleIndex ?? op.defaultModule,
            moduleLevel: 3,
            buffs: {},
            conditionals,
            allCond: true,
        },
    };
}

function cryptoRandomId(): string {
    if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
        return crypto.randomUUID();
    }
    return `i-${Math.random().toString(36).slice(2)}-${Date.now().toString(36)}`;
}

/**
 * Pick the first palette color not currently in use. Falls back to
 * `getInstanceColor(N)` (which wraps the palette) once every color is taken.
 */
function pickAvailableColor(used: ReadonlySet<string>, instanceCount: number): string {
    for (const color of CHART_PALETTE) {
        if (!used.has(color)) return color;
    }
    return getInstanceColor(instanceCount);
}

export function dpsReducer(state: IDpsState, action: DpsAction): IDpsState {
    switch (action.type) {
        case "ADD_INSTANCE": {
            const usedColors = new Set(state.instances.map((i) => i.color));
            const inst = createInstance(action.op, pickAvailableColor(usedColors, state.instances.length));
            return { ...state, instances: [...state.instances, inst] };
        }
        case "DUPLICATE_INSTANCE": {
            const src = state.instances.find((i) => i.uid === action.uid);
            if (!src) return state;
            const usedColors = new Set(state.instances.map((i) => i.color));
            const clone: IDpsInstance = {
                ...src,
                uid: cryptoRandomId(),
                color: pickAvailableColor(usedColors, state.instances.length),
                config: {
                    ...src.config,
                    buffs: { ...src.config.buffs },
                    conditionals: { ...src.config.conditionals },
                },
            };
            return { ...state, instances: [...state.instances, clone] };
        }
        case "REMOVE_INSTANCE":
            return { ...state, instances: state.instances.filter((i) => i.uid !== action.uid) };
        case "UPDATE_CONFIG":
            return {
                ...state,
                instances: state.instances.map((i) => (i.uid === action.uid ? { ...i, config: { ...i.config, ...action.patch } } : i)),
            };
        case "UPDATE_BUFFS":
            return {
                ...state,
                instances: state.instances.map((i) =>
                    i.uid === action.uid
                        ? {
                              ...i,
                              config: { ...i.config, buffs: { ...i.config.buffs, ...action.patch } },
                          }
                        : i,
                ),
            };
        case "TOGGLE_CONDITIONAL":
            return {
                ...state,
                instances: state.instances.map((i) =>
                    i.uid === action.uid
                        ? {
                              ...i,
                              config: {
                                  ...i.config,
                                  conditionals: { ...i.config.conditionals, [action.key]: action.value },
                              },
                          }
                        : i,
                ),
            };
        case "TOGGLE_VISIBILITY":
            return {
                ...state,
                instances: state.instances.map((i) => (i.uid === action.uid ? { ...i, visible: !i.visible } : i)),
            };
        case "TOGGLE_COLLAPSED":
            return {
                ...state,
                instances: state.instances.map((i) => (i.uid === action.uid ? { ...i, collapsed: !i.collapsed } : i)),
            };
        case "SET_ALL_COLLAPSED":
            return {
                ...state,
                instances: state.instances.map((i) => (i.collapsed === action.collapsed ? i : { ...i, collapsed: action.collapsed })),
            };
        case "SET_ENEMY":
            return { ...state, enemy: { ...state.enemy, ...action.patch } };
        case "SET_SHRED":
            return { ...state, enemy: { ...state.enemy, shred: { ...state.enemy.shred, ...action.patch } } };
        case "SET_AXIS":
            return { ...state, xAxis: action.axis };
        case "SET_METRIC":
            return { ...state, yMetric: action.metric };
        case "SET_SWEEP":
            return {
                ...state,
                sweep: {
                    ...state.sweep,
                    [action.axis]: { ...state.sweep[action.axis], ...action.patch },
                },
            };
        case "REORDER_INSTANCE": {
            const idx = state.instances.findIndex((i) => i.uid === action.uid);
            if (idx === -1) return state;
            const target = action.direction === "up" ? idx - 1 : idx + 1;
            if (target < 0 || target >= state.instances.length) return state;
            const next = [...state.instances];
            const [moved] = next.splice(idx, 1);
            next.splice(target, 0, moved);
            return { ...state, instances: next };
        }
        case "RESET_INSTANCES":
            return { ...state, instances: [] };
        case "REFRESH_OPS": {
            let dirty = false;
            const next = state.instances.map((inst) => {
                const fresh = action.freshOps.get(inst.op.id);
                if (!fresh || fresh === inst.op) return inst;
                dirty = true;
                return { ...inst, op: fresh };
            });
            return dirty ? { ...state, instances: next } : state;
        }
        case "HYDRATE":
            return action.state;
    }
}

export function instanceToRequest(inst: IDpsInstance, enemy: IEnemyConfig, axisOverride?: { defense?: number; res?: number }): IDpsCalculateRequest {
    const cfg = inst.config;
    return {
        operatorId: inst.op.id,
        promotion: cfg.promotion,
        level: cfg.level,
        potential: cfg.potential,
        trust: cfg.trust,
        skillIndex: cfg.skillIndex,
        masteryLevel: cfg.masteryLevel,
        moduleIndex: cfg.moduleIndex,
        moduleLevel: cfg.moduleIndex > 0 ? cfg.moduleLevel : undefined,
        defense: axisOverride?.defense ?? enemy.defense,
        res: axisOverride?.res ?? enemy.res,
        targets: enemy.targets,
        spBoost: enemy.spBoost,
        buffs: stripEmpty(cfg.buffs as Record<string, number | undefined>) as IDpsCalculateRequest["buffs"],
        shred: stripEmpty(enemy.shred as Record<string, number | undefined>) as IDpsCalculateRequest["shred"],
        conditionals: cfg.conditionals,
        allCond: cfg.allCond,
    };
}

function stripEmpty(obj: Record<string, number | undefined>): Record<string, number> | undefined {
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
