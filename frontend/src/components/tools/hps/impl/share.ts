import { decodeInstances, decodeSweep, encodeInstance, type ISharedInstance, isFiniteNumber, packShare, pickIndex, resolveSharedInstances, unpackShare } from "#/components/tools/shared/shareCodec";
import type { IOperatorListEntry } from "#/components/tools/shared/types";
import { INITIAL_STATE } from "./state";
import type { HpsXAxis, HpsYMetric, IHpsBuffConfig, IHpsState } from "./types";

/** A decoded HPS link: the page state with each card's operator still an id. */
export interface IHpsShare extends Omit<IHpsState, "instances"> {
    instances: ISharedInstance[];
    /** Card entries dropped as malformed. */
    invalid: number;
}

const X_AXES: readonly HpsXAxis[] = ["targets", "atk", "aspd"];
const Y_METRICS: readonly HpsYMetric[] = ["skill_hps", "base_hps", "avg_hps"];

/**
 * Body layout (version 1): `{ i: cards, b: [atk, flatAtk, aspd, fragile, targets, spBoost],
 * x, y, s: [[targets sweep], [atk sweep], [aspd sweep]] }`. Unset team buffs travel as null.
 */
export function encodeHpsShare(state: IHpsState): string {
    const { buffs, sweep } = state;
    return packShare("hps", {
        i: state.instances.map(encodeInstance),
        b: [buffs.buffs.atk ?? null, buffs.buffs.flatAtk ?? null, buffs.buffs.aspd ?? null, buffs.buffs.fragile ?? null, buffs.targets, buffs.spBoost],
        x: X_AXES.indexOf(state.xAxis),
        y: Y_METRICS.indexOf(state.yMetric),
        s: X_AXES.map((a) => [sweep[a].min, sweep[a].max, sweep[a].steps]),
    });
}

/** Null for anything that is not a version-1 HPS link with a valid team-buff block. */
export function decodeHpsShare(text: string): IHpsShare | null {
    const body = unpackShare("hps", text);
    if (!body || typeof body !== "object" || Array.isArray(body)) return null;
    const b = body as Record<string, unknown>;
    const cards = decodeInstances(b.i);
    const buffs = decodeBuffs(b.b);
    if (!cards || !buffs) return null;
    const sweeps = Array.isArray(b.s) ? b.s : [];
    return {
        instances: cards.instances,
        invalid: cards.invalid,
        buffs,
        xAxis: pickIndex(X_AXES, b.x) ?? INITIAL_STATE.xAxis,
        yMetric: pickIndex(Y_METRICS, b.y) ?? INITIAL_STATE.yMetric,
        sweep: {
            targets: decodeSweep(sweeps[0]) ?? { ...INITIAL_STATE.sweep.targets },
            atk: decodeSweep(sweeps[1]) ?? { ...INITIAL_STATE.sweep.atk },
            aspd: decodeSweep(sweeps[2]) ?? { ...INITIAL_STATE.sweep.aspd },
        },
    };
}

export function resolveHpsShare(share: IHpsShare, operators: readonly IOperatorListEntry[]): { state: IHpsState; missing: number } {
    const { instances, missing } = resolveSharedInstances(share.instances, operators);
    const { invalid: _invalid, ...rest } = share;
    return { state: { ...rest, instances }, missing };
}

function decodeBuffs(raw: unknown): IHpsBuffConfig | null {
    if (!Array.isArray(raw) || raw.length !== 6) return null;
    const [atk, flatAtk, aspd, fragile, targets, spBoost] = raw as unknown[];
    if (![atk, flatAtk, aspd, fragile].every((v) => v === null || isFiniteNumber(v))) return null;
    if (!isFiniteNumber(targets) || !isFiniteNumber(spBoost)) return null;
    const values = { atk, flatAtk, aspd, fragile } as Record<string, number | null>;
    const buffs: IHpsBuffConfig["buffs"] = {};
    for (const [k, v] of Object.entries(values)) if (v !== null) buffs[k as keyof typeof buffs] = v;
    return { buffs, targets: Math.max(1, Math.round(targets)), spBoost };
}
