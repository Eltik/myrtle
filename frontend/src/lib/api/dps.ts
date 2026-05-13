import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

/**
 * One conditional damage toggle exposed by an operator (e.g. "Apply S2 ATK
 * buff" or "Trait active vs. flying enemies"). The `type` field maps to the
 * backend's `cond_type` and corresponds to one of the `conditionals` keys on
 * {@link IDpsCalculateConditionals}.
 */
export interface IDpsConditionalInfo {
    /** One of: "trait", "talent", "talent2", "skill", "module". */
    conditionalType: string;
    name: string;
    /** Recommended default - match this when initialising the form. */
    default: boolean;
    /** Skill indices (1-indexed: 1=S1, 2=S2, 3=S3) where this conditional applies. Empty = all. */
    skills: number[];
    /** Module indices where this conditional applies. Empty = all. */
    modules: number[];
}

/** One row from `GET /dps/operators` - every operator the engine can compute. */
export interface IDpsOperatorListEntry {
    /** Internal char_id, e.g. "char_002_amiya". */
    id: string;
    /** Display name. */
    name: string;
    /** Skill indices the engine has formulas for. 1-indexed (1=S1, 2=S2, 3=S3). */
    availableSkills: number[];
    /** Module indices the engine has formulas for. */
    availableModules: number[];
    /** Engine-recommended default skill (1-indexed). */
    defaultSkill: number;
    /** Engine-recommended default module (0 = no module). */
    defaultModule: number;
    conditionals: IDpsConditionalInfo[];
}

/**
 * External buffs applied on top of the operator's own stats - matches the
 * backend's `OperatorBuffs`. All fields are optional.
 */
export interface IDpsCalculateBuffs {
    /** ATK% buff as a decimal (e.g. 0.4 = +40%). */
    atk?: number;
    /** Flat ATK buff (e.g. 102 = +102 ATK). */
    flatAtk?: number;
    /** ASPD buff in points (e.g. 52 = +52 ASPD). */
    aspd?: number;
    /** Fragile / damage taken buff as a decimal (e.g. 0.3 = +30% damage taken). */
    fragile?: number;
}

/**
 * DEF/RES shred applied to the enemy - matches `OperatorShred`. The percent
 * fields are integers (e.g. `def: 40` = -40% DEF), not decimals.
 */
export interface IDpsCalculateShred {
    /** DEF shred as integer percent (40 = -40%). */
    def?: number;
    /** Flat DEF shred (40 = -40 DEF). */
    defFlat?: number;
    /** RES shred as integer percent (40 = -40%). */
    res?: number;
    /** Flat RES shred (10 = -10 RES). */
    resFlat?: number;
}

/**
 * Per-source damage toggles. `null`/`undefined` means "use the operator's
 * default for this conditional"; `false` explicitly disables it. See
 * {@link IDpsConditionalInfo} for the legal keys per operator.
 */
export interface IDpsCalculateConditionals {
    traitDamage?: boolean;
    talentDamage?: boolean;
    talent2Damage?: boolean;
    skillDamage?: boolean;
    moduleDamage?: boolean;
}

/**
 * Body for `POST /dps/calculate`. Only `operatorId` is required - every other
 * field has a backend-side default that mirrors the Python reference.
 */
export interface IDpsCalculateRequest {
    operatorId: string;

    // Operator config
    /** 0=E0, 1=E1, 2=E2. Omit / -1 = max for rarity. */
    promotion?: number;
    /** Level within the chosen promotion. Omit / <=0 = max for promotion+rarity. */
    level?: number;
    /** Potential 1-6. Omit = formula default (usually 1). */
    potential?: number;
    /** Trust 0-100. Omit = 100. */
    trust?: number;
    /** 1-indexed skill (1=S1, 2=S2, 3=S3). 0 = no skill / basic attacks. */
    skillIndex?: number;
    /** Mastery level 1-3. Omit = M3 / max for promotion. */
    masteryLevel?: number;
    /** Module sequence number from {@link IDpsOperatorListEntry.availableModules}. 0 = no module. */
    moduleIndex?: number;
    /** Module level 1-3. Omit = 3. Auto-clamped server-side based on trust. */
    moduleLevel?: number;

    // Enemy
    /** Enemy DEF before shreds. Defaults to 0. */
    defense?: number;
    /** Enemy RES (0-100) before shreds. Defaults to 0. */
    res?: number;

    // Buffs & shreds
    buffs?: IDpsCalculateBuffs;
    shred?: IDpsCalculateShred;

    // Targets
    /** Number of targets hit by AoE skills. Defaults to 1. Clamped to >=1. */
    targets?: number;
    /** Bonus SP/sec from external sources (e.g. Ptilopsis). Decimal, defaults to 0. */
    spBoost?: number;

    // Conditionals
    conditionals?: IDpsCalculateConditionals;
    /** When false, every unspecified conditional is forced off. Defaults to true. */
    allCond?: boolean;
}

/**
 * Result from `POST /dps/calculate`. Note the backend returns these in
 * snake_case (the {@link DpsResult} struct has no rename attribute).
 */
export interface IDpsCalculateResponse {
    /** DPS while the skill is active, including all buffs and conditionals. */
    skill_dps: number;
    /**
     * If the skill has a finite duration: `skill_dps * skill_duration`.
     * For zero-duration / passive skills: equal to `skill_dps`.
     */
    total_damage: number;
    /**
     * Long-run DPS averaged over a full skill+downtime cycle (skill_dps during
     * uptime, basic-attack DPS during SP recharge). Falls back to `skill_dps`
     * when the skill has no duration or no SP cost.
     */
    average_dps: number;
}

export const getDpsOperatorsFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/dps/operators");
    if (!res.ok) throw new Error(`Failed to load DPS operators: ${res.status}`);
    return (await res.json()) as IDpsOperatorListEntry[];
});

export function dpsOperatorsQueryOptions() {
    return queryOptions({
        queryKey: ["dps", "operators"],
        queryFn: () => getDpsOperatorsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

export const calculateDpsFn = createServerFn({ method: "POST" })
    .inputValidator((data: IDpsCalculateRequest) => data)
    .handler(async ({ data }) => {
        const res = await backendFetch("/dps/calculate", {
            method: "POST",
            body: JSON.stringify(data),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to calculate DPS: ${res.status}`);
        }
        return (await res.json()) as IDpsCalculateResponse;
    });

/**
 * Stable serialisation for use in a react-query key. Recursively sorts keys so
 * `{ a: 1, b: 2 }` and `{ b: 2, a: 1 }` hash identically - and nested objects
 * (conditionals, buffs, shred) are included in full rather than stripped by
 * `JSON.stringify`'s replacer-array filter.
 */
function dpsRequestKey(req: IDpsCalculateRequest): string {
    return stableStringify(req);
}

function stableStringify(value: unknown): string {
    if (value === null || typeof value !== "object") return JSON.stringify(value) ?? "null";
    if (Array.isArray(value)) return `[${value.map(stableStringify).join(",")}]`;
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${stableStringify(obj[k])}`).join(",")}}`;
}

/**
 * Cache-keyed variant for parametric DPS calculation. Useful when the same
 * (operator, config) tuple is rendered repeatedly - e.g. plotting a curve
 * across skill levels - and the caller wants react-query deduplication.
 *
 * For one-off "user clicked Recalculate" flows, prefer `useMutation` against
 * {@link calculateDpsFn} directly so retries and loading state are explicit.
 */
export function dpsCalculateQueryOptions(req: IDpsCalculateRequest) {
    return queryOptions({
        queryKey: ["dps", "calculate", req.operatorId, dpsRequestKey(req)],
        queryFn: () => calculateDpsFn({ data: req }),
        enabled: Boolean(req.operatorId),
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

/**
 * Convenience: pull the engine-recommended defaults for an operator into a
 * ready-to-submit request body. Conditionals use each entry's `default` value.
 */
export function buildDefaultDpsRequest(op: IDpsOperatorListEntry): IDpsCalculateRequest {
    const conditionals: IDpsCalculateConditionals = {};
    for (const c of op.conditionals) {
        switch (c.conditionalType) {
            case "trait":
                conditionals.traitDamage = c.default;
                break;
            case "talent":
                conditionals.talentDamage = c.default;
                break;
            case "talent2":
                conditionals.talent2Damage = c.default;
                break;
            case "skill":
                conditionals.skillDamage = c.default;
                break;
            case "module":
                conditionals.moduleDamage = c.default;
                break;
        }
    }

    return {
        operatorId: op.id,
        skillIndex: op.defaultSkill,
        moduleIndex: op.defaultModule,
        trust: 100,
        targets: 1,
        defense: 0,
        res: 0,
        buffs: {},
        shred: {},
        conditionals,
        allCond: true,
    };
}
