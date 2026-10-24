import { deflateSync, inflateSync, strFromU8, strToU8 } from "fflate";
import { decodeZones, encodeZones } from "../dps/impl/zones";
import { CHART_PALETTE } from "./constants";
import { createInstance, pickAvailableColor } from "./instance";
import { MAX_SKILL_RANK } from "./skill";
import type { IBuildConfig, ICalcBuffs, ICalcConditionals, IInstance, IOperatorListEntry } from "./types";

/**
 * Shareable calculator links: the whole configuration in one `?s=` search param.
 *
 * Wire format: `[SHARE_VERSION, kind, body]` as JSON, deflated, base64url. A link
 * carrying another version or kind is ignored rather than guessed at; bump the
 * version whenever a positional layout below changes.
 */
export const SHARE_VERSION = 1;

export type ShareKind = "dps" | "hps";

/** Longest param accepted. A full 8-card DPS setup packs to about 300 characters. */
const MAX_PARAM_LENGTH = 8192;

export function packShare(kind: ShareKind, body: unknown): string {
    return toBase64Url(deflateSync(strToU8(JSON.stringify([SHARE_VERSION, kind, body])), { level: 9 }));
}

/** The body of a well-formed link of this version and kind, else null. Never throws. */
export function unpackShare(kind: ShareKind, text: string): unknown {
    if (text.length === 0 || text.length > MAX_PARAM_LENGTH || !/^[A-Za-z0-9_-]+$/.test(text)) return null;
    try {
        const parsed = JSON.parse(strFromU8(inflateSync(fromBase64Url(text)))) as unknown;
        if (!Array.isArray(parsed) || parsed.length !== 3 || parsed[0] !== SHARE_VERSION || parsed[1] !== kind) return null;
        return parsed[2];
    } catch {
        return null;
    }
}

function toBase64Url(bytes: Uint8Array): string {
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

function fromBase64Url(text: string): Uint8Array {
    const b64 = text.replaceAll("-", "+").replaceAll("_", "/");
    const bin = atob(b64 + "=".repeat((4 - (b64.length % 4)) % 4));
    return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}

/** One card as read from a link: the operator by id only, resolved against the live list later. */
export interface ISharedInstance {
    opId: string;
    /** Absent when the card's colour is not in the palette; a free one is picked on resolve. */
    color?: string;
    visible: boolean;
    config: IBuildConfig;
}

const CONDITIONAL_ORDER: readonly (keyof ICalcConditionals)[] = ["traitDamage", "talentDamage", "talent2Damage", "skillDamage", "moduleDamage"];
const BUFF_ORDER: readonly (keyof ICalcBuffs)[] = ["atk", "flatAtk", "aspd", "fragile"];

/**
 * Positional card layout (version 1):
 * `[opId, paletteIndex, visible, promotion, level, potential, trust, skillIndex, skillRank, moduleIndex, moduleLevel, buffs, condSet, condOn, allCond]`,
 * and a sixteenth entry when a DPS card has its own zones: the zone vector (`encodeZones`).
 * A 15-entry tuple is a card with no zones.
 * `promotion` / `level` and each buff are null when unset. Conditionals are two bit masks
 * over {@link CONDITIONAL_ORDER}: which flags are set at all, and which are on. An unset
 * flag is not the same as `false`: the engine reads a missing flag as on.
 */
export function encodeInstance(inst: IInstance): unknown[] {
    const cfg = inst.config;
    let condSet = 0;
    let condOn = 0;
    CONDITIONAL_ORDER.forEach((key, bit) => {
        const v = cfg.conditionals[key];
        if (v === undefined) return;
        condSet |= 1 << bit;
        if (v) condOn |= 1 << bit;
    });
    const tuple: unknown[] = [inst.op.id, CHART_PALETTE.indexOf(inst.color), inst.visible ? 1 : 0, cfg.promotion ?? null, cfg.level ?? null, cfg.potential, cfg.trust, cfg.skillIndex, cfg.skillRank, cfg.moduleIndex, cfg.moduleLevel, BUFF_ORDER.map((k) => cfg.buffs[k] ?? null), condSet, condOn, cfg.allCond ? 1 : 0];
    if (cfg.zones) tuple.push(encodeZones(cfg.zones));
    return tuple;
}

/** Null when the tuple is malformed; numbers are clamped to the ranges the controls allow. */
export function decodeInstance(raw: unknown): ISharedInstance | null {
    if (!Array.isArray(raw) || (raw.length !== 15 && raw.length !== 16)) return null;
    const [opId, paletteIndex, visible, promotion, level, potential, trust, skillIndex, skillRank, moduleIndex, moduleLevel, buffs, condSet, condOn, allCond, rawZones] = raw as unknown[];
    if (typeof opId !== "string" || opId.length === 0 || opId.length > 64) return null;
    if (!isInt(paletteIndex) || !isInt(potential) || !isInt(trust) || !isInt(skillIndex) || !isInt(skillRank) || !isInt(moduleIndex) || !isInt(moduleLevel) || !isInt(condSet) || !isInt(condOn)) return null;
    if ((promotion !== null && !isInt(promotion)) || (level !== null && !isInt(level))) return null;
    if (!Array.isArray(buffs) || buffs.length !== BUFF_ORDER.length || !buffs.every((b) => b === null || isFiniteNumber(b))) return null;
    const zones = raw.length === 16 ? decodeZones(rawZones) : undefined;
    if (zones === null) return null;

    const conditionals: ICalcConditionals = {};
    CONDITIONAL_ORDER.forEach((key, bit) => {
        if (condSet & (1 << bit)) conditionals[key] = Boolean(condOn & (1 << bit));
    });
    const decodedBuffs: ICalcBuffs = {};
    BUFF_ORDER.forEach((key, i) => {
        const v = buffs[i] as number | null;
        if (v !== null) decodedBuffs[key] = v;
    });

    const config: IBuildConfig = {
        potential: clamp(potential, 1, 6),
        trust: clamp(trust, 0, 100),
        skillIndex: clamp(skillIndex, 0, 3),
        skillRank: clamp(skillRank, 1, MAX_SKILL_RANK),
        moduleIndex: clamp(moduleIndex, 0, 9),
        moduleLevel: clamp(moduleLevel, 1, 3),
        buffs: decodedBuffs,
        conditionals,
        allCond: allCond === 1,
    };
    if (promotion !== null) config.promotion = clamp(promotion, 0, 2);
    if (level !== null) config.level = clamp(level, 1, 90);
    if (zones) config.zones = zones;

    const color = CHART_PALETTE[paletteIndex];
    return { opId, ...(color ? { color } : {}), visible: visible === 1, config };
}

/**
 * Decode a card list, dropping malformed entries. Null when `raw` is not a list at all.
 * `invalid` counts the dropped entries so the page can say so.
 */
export function decodeInstances(raw: unknown, limit = 32): { instances: ISharedInstance[]; invalid: number } | null {
    if (!Array.isArray(raw)) return null;
    const instances: ISharedInstance[] = [];
    let invalid = 0;
    for (const entry of raw.slice(0, limit)) {
        const decoded = decodeInstance(entry);
        if (decoded) instances.push(decoded);
        else invalid++;
    }
    return { instances, invalid: invalid + Math.max(0, raw.length - limit) };
}

/** Attach live operator entries; a card whose operator is no longer in the list is dropped and counted. */
export function resolveSharedInstances(shared: readonly ISharedInstance[], operators: readonly IOperatorListEntry[]): { instances: IInstance[]; missing: number } {
    const byId = new Map(operators.map((op) => [op.id, op]));
    const instances: IInstance[] = [];
    const used = new Set<string>();
    let missing = 0;
    for (const s of shared) {
        const op = byId.get(s.opId);
        if (!op) {
            missing++;
            continue;
        }
        const color = s.color && !used.has(s.color) ? s.color : pickAvailableColor(used, instances.length);
        used.add(color);
        const inst = createInstance(op, color, s.config);
        instances.push({ ...inst, visible: s.visible });
    }
    return { instances, missing };
}

/** `[min, max, steps]`, validated; null when malformed. */
export function decodeSweep(raw: unknown): { min: number; max: number; steps: number } | null {
    if (!Array.isArray(raw) || raw.length !== 3 || !raw.every(isFiniteNumber)) return null;
    const [min, max, steps] = raw as number[];
    if (min < 0 || max <= min || !Number.isInteger(steps) || steps < 2 || steps > 101) return null;
    return { min, max, steps };
}

/** `options[index]` for an integer index in range, else undefined. */
export function pickIndex<T>(options: readonly T[], index: unknown): T | undefined {
    return Number.isInteger(index) && (index as number) >= 0 ? options[index as number] : undefined;
}

export function isFiniteNumber(v: unknown): v is number {
    return typeof v === "number" && Number.isFinite(v);
}

function isInt(v: unknown): v is number {
    return typeof v === "number" && Number.isInteger(v);
}

function clamp(v: number, lo: number, hi: number): number {
    return Math.min(hi, Math.max(lo, v));
}
