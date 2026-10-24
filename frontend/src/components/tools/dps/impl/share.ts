import { decodeInstances, decodeSweep, encodeInstance, type ISharedInstance, isFiniteNumber, packShare, pickIndex, resolveSharedInstances, unpackShare } from "#/components/tools/shared/shareCodec";
import type { IBuildConfig, IOperatorListEntry } from "#/components/tools/shared/types";
import type { RankPreset } from "#/types/generated/RankPreset";
import { normaliseRankSettings } from "./rank";
import { DEFAULT_ENEMY, DEFAULT_RANK, DEFAULT_TEAM, INITIAL_STATE } from "./state";
import type { DpsView, IDpsState, IEnemyConfig, IRankSettings, ISecondaryConfig, ITeamBuffs, RankSortKey, XAxisKind, YMetric } from "./types";
import { decodeZones, encodeZones, legacyBuffsToZones } from "./zones";

/** A decoded DPS link: the page state with each card's operator still an id. */
export interface IDpsShare extends Omit<IDpsState, "instances"> {
    instances: ISharedInstance[];
    /** Card entries dropped as malformed. */
    invalid: number;
}

const X_AXES: readonly XAxisKind[] = ["defense", "res"];
const Y_METRICS: readonly YMetric[] = ["skill_dps", "average_dps", "total_damage"];
const VIEWS: readonly DpsView[] = ["compare", "rank"];
const PRESETS: readonly RankPreset[] = ["p1Max", "p6Max", "default"];
const SORTS: readonly RankSortKey[] = ["skillDps", "averageDps", "totalDamage", "retention"];
const MAX_SUPPORTERS = 64;

/**
 * Body layout (version 1): `{ i: cards, e: [defense, res, targets], q: [secondaryOn, def, res,
 * count], c: cycleV2, z: team zone vector, p: supporter ids, n: [promotion, skillLevel,
 * potential, moduleLevel], x, y, s: [[defense sweep], [res sweep]], m, v: view, r: [preset,
 * sort, professions, rarities, damageTypes] }`. Cards are `encodeInstance` tuples, with
 * their own zone vector as a sixteenth entry when they have one.
 */
export function encodeDpsShare(state: IDpsState): string {
    const { enemy, sweep, team, rank } = state;
    const sec = enemy.secondary;
    return packShare("dps", {
        i: state.instances.map(encodeInstance),
        e: [enemy.defense, enemy.res, enemy.targets],
        q: [sec.enabled ? 1 : 0, sec.def, sec.res, sec.count],
        c: enemy.cycleModel === "v2" ? 1 : 0,
        z: encodeZones(team.zones),
        p: team.supporters,
        n: [team.investment.promotion, team.investment.skillLevel, team.investment.potential, team.investment.moduleLevel],
        x: X_AXES.indexOf(state.xAxis),
        y: Y_METRICS.indexOf(state.yMetric),
        s: X_AXES.map((a) => [sweep[a].min, sweep[a].max, sweep[a].steps]),
        m: state.markers ? 1 : 0,
        v: VIEWS.indexOf(state.view),
        r: [PRESETS.indexOf(rank.preset), SORTS.indexOf(rank.sortBy), rank.professions, rank.rarities, rank.damageTypes],
    });
}

/** Null for anything that is not a version-1 DPS link with a valid enemy block. */
export function decodeDpsShare(text: string): IDpsShare | null {
    const body = unpackShare("dps", text);
    if (!body || typeof body !== "object" || Array.isArray(body)) return null;
    const b = body as Record<string, unknown>;
    const cards = decodeInstances(b.i);
    const head = decodeHead(b);
    if (!cards || !head) return null;
    const sweeps = Array.isArray(b.s) ? b.s : [];
    return {
        // A card carrying legacy External buffs (a hand-off that slipped through) gets them as its own zones.
        instances: cards.instances.map((inst) => ({ ...inst, config: migrateCardBuffs(inst.config) })),
        invalid: cards.invalid,
        enemy: head.enemy,
        team: head.team,
        xAxis: pickIndex(X_AXES, b.x) ?? INITIAL_STATE.xAxis,
        yMetric: pickIndex(Y_METRICS, b.y) ?? INITIAL_STATE.yMetric,
        sweep: {
            defense: decodeSweep(sweeps[0]) ?? { ...INITIAL_STATE.sweep.defense },
            res: decodeSweep(sweeps[1]) ?? { ...INITIAL_STATE.sweep.res },
        },
        markers: b.m === undefined ? INITIAL_STATE.markers : b.m === 1,
        view: pickIndex(VIEWS, b.v) ?? INITIAL_STATE.view,
        rank: decodeRank(b.r),
    };
}

export function resolveDpsShare(share: IDpsShare, operators: readonly IOperatorListEntry[]): { state: IDpsState; missing: number } {
    const { instances, missing } = resolveSharedInstances(share.instances, operators);
    const { invalid: _invalid, ...rest } = share;
    return { state: { ...rest, instances }, missing };
}

/** Legacy `buffs` -> the card's own zones, unless it already has zones. Buffs end up empty either way. */
export function migrateCardBuffs(config: IBuildConfig): IBuildConfig {
    const hasBuffs = Object.values(config.buffs ?? {}).some((v) => typeof v === "number" && v !== 0);
    if (!hasBuffs) return config.buffs && Object.keys(config.buffs).length === 0 ? config : { ...config, buffs: {} };
    const zones = config.zones ?? legacyBuffsToZones(config.buffs);
    return { ...config, buffs: {}, ...(zones ? { zones } : {}) };
}

function decodeHead(b: Record<string, unknown>): { enemy: IEnemyConfig; team: ITeamBuffs } | null {
    const e = b.e;
    if (!Array.isArray(e) || e.length !== 3 || !e.every(isFiniteNumber)) return null;
    const [defense, res, targets] = e as number[];
    const zones = b.z === undefined ? DEFAULT_TEAM.zones : decodeZones(b.z);
    if (!zones) return null;
    return {
        enemy: {
            defense: Math.max(0, defense),
            res: Math.min(100, Math.max(0, res)),
            targets: Math.max(1, Math.round(targets)),
            secondary: decodeSecondary(b.q),
            cycleModel: b.c === 1 ? "v2" : "legacy",
        },
        team: {
            zones,
            supporters: Array.isArray(b.p) ? [...new Set(b.p.filter((id): id is string => typeof id === "string" && id.length > 0 && id.length <= 64))].slice(0, MAX_SUPPORTERS) : [],
            investment: decodeInvestment(b.n),
        },
    };
}

function decodeSecondary(raw: unknown): ISecondaryConfig {
    if (!Array.isArray(raw) || raw.length !== 4 || !raw.every(isFiniteNumber)) return { ...DEFAULT_ENEMY.secondary };
    const [on, def, res, count] = raw as number[];
    return { enabled: on === 1, def: Math.max(0, def), res: Math.min(100, Math.max(0, res)), count: Math.min(49, Math.max(1, Math.round(count))) };
}

function decodeInvestment(raw: unknown): ITeamBuffs["investment"] {
    if (!Array.isArray(raw) || raw.length !== 4 || !raw.every((v) => Number.isInteger(v))) return { ...DEFAULT_TEAM.investment };
    const [promotion, skillLevel, potential, moduleLevel] = raw as number[];
    return { promotion: clamp(promotion, 0, 2), skillLevel: clamp(skillLevel, 1, 10), potential: clamp(potential, 1, 6), moduleLevel: clamp(moduleLevel, 0, 3) };
}

function decodeRank(raw: unknown): IRankSettings {
    if (!Array.isArray(raw) || raw.length !== 5) return DEFAULT_RANK;
    const [preset, sortBy, professions, rarities, damageTypes] = raw as unknown[];
    return normaliseRankSettings({ preset: pickIndex(PRESETS, preset), sortBy: pickIndex(SORTS, sortBy), professions, rarities, damageTypes }, DEFAULT_RANK);
}

function clamp(v: number, lo: number, hi: number): number {
    return Math.min(hi, Math.max(lo, v));
}
