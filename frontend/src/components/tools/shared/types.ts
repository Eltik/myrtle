/** Type contracts for the operator-build calculators. */

/** One conditional toggle exposed by an operator (trait/talent/skill/module). */
export interface IConditionalInfo {
    /** One of: "trait", "talent", "talent2", "skill", "module". */
    conditionalType: string;
    name: string;
    /** Recommended default - match this when initialising the form. */
    default: boolean;
    /** Skill indices (1-indexed) where this conditional applies. Empty = all. */
    skills: number[];
    /** Module indices where this conditional applies. Empty = all. */
    modules: number[];
}

/** One row from a `/{dps,hps}/operators` listing. */
export interface IOperatorListEntry {
    id: string;
    name: string;
    availableSkills: number[];
    availableModules: number[];
    defaultSkill: number;
    defaultModule: number;
    conditionals: IConditionalInfo[];
}

/** External buffs applied on top of the operator's own stats. */
export interface ICalcBuffs {
    /** ATK% buff as a decimal (0.4 = +40%). */
    atk?: number;
    /** Flat ATK buff. */
    flatAtk?: number;
    /** ASPD buff in points. */
    aspd?: number;
    /** Fragile / heal-amp as a decimal (0.3 = +30%). */
    fragile?: number;
}

/** Per-source conditional toggles. Keys map to the backend's `conditionals`. */
export interface ICalcConditionals {
    traitDamage?: boolean;
    talentDamage?: boolean;
    talent2Damage?: boolean;
    skillDamage?: boolean;
    moduleDamage?: boolean;
}

/** Shared body fields for `POST /{dps,hps}/calculate`. */
export interface ICalcRequestBase {
    operatorId: string;
    promotion?: number;
    level?: number;
    potential?: number;
    trust?: number;
    /** 1-indexed skill (1=S1, 2=S2, 3=S3). */
    skillIndex?: number;
    /** Mastery 1-3 (M1-M3). Used for skill ranks 8-10. */
    masteryLevel?: number;
    /** Explicit pre-mastery skill level 1-7. Overrides `masteryLevel` server-side. */
    skillLevel?: number;
    moduleIndex?: number;
    moduleLevel?: number;
    buffs?: ICalcBuffs;
    targets?: number;
    spBoost?: number;
    conditionals?: ICalcConditionals;
    allCond?: boolean;
}

/**
 * One operator's build configuration in a calculator instance. `skillRank`
 * unifies skill level and mastery into a single 1-10 scale: 1-7 are skill
 * levels, 8/9/10 are masteries M1/M2/M3. {@link ./skill} maps it to the
 * backend's `skillLevel`/`masteryLevel` fields.
 */
export interface IBuildConfig {
    promotion?: number;
    level?: number;
    potential: number;
    trust: number;
    skillIndex: number;
    skillRank: number;
    moduleIndex: number;
    moduleLevel: number;
    /** Per-instance external buffs (DPS). Unused by HPS, which buffs globally. */
    buffs: ICalcBuffs;
    conditionals: ICalcConditionals;
    allCond: boolean;
}

export interface IInstance<TOp extends IOperatorListEntry = IOperatorListEntry> {
    uid: string;
    op: TOp;
    color: string;
    visible: boolean;
    collapsed: boolean;
    config: IBuildConfig;
}

export interface ISweepRange {
    min: number;
    max: number;
    steps: number;
}

export interface ICurvePoint {
    x: number;
    /** Map of instance uid → metric value at this X (null while loading). */
    [uid: string]: number | null;
}

/** Per-axis sweep-input granularity for the range controls. */
export interface IAxisInput {
    step: number;
    maxBound: number;
    integer: boolean;
    unit: string;
}

/** A KPI/result column shown per operator in the snapshot panel. */
export interface IMetricColumn {
    /** Response key, e.g. "skill_dps" / "avg_hps". */
    key: string;
    label: string;
    /** Optional per-row hint (e.g. explaining a 0 value). */
    hint?: (data: Record<string, number>) => string | undefined;
}
