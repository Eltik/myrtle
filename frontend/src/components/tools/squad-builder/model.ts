/**
 * The Squad Builder's working copy and the pure functions around it: build
 * resolution against the synced roster, the analyze request, and the share
 * link. No React here; every component imports from this module.
 *
 * Wire types come from the backend (`#/types/generated`). A `SquadMember`
 * with `build: null` follows the roster: its promotion, level, potential,
 * skill level, mastery and module rank are read from the signed-in user's
 * synced roster when the page loads, and from a full build when the operator
 * is not owned. A non-null `build` is a build set by hand.
 */

import type { CoverageRowKey } from "#/types/generated/CoverageRowKey";
import type { RosterEntry } from "#/types/generated/RosterEntry";
import type { SquadAnalyzeRequest } from "#/types/generated/SquadAnalyzeRequest";
import type { SquadBuild } from "#/types/generated/SquadBuild";
import type { SquadMember } from "#/types/generated/SquadMember";
import type { SquadMemberBuild } from "#/types/generated/SquadMemberBuild";
import type { Unlock } from "#/types/generated/Unlock";
import type { IOperatorIndexEntry } from "#/types/operators";
import { DPS_SQUADMATE_LIMIT, NAME_MAX_CHARS, SQUAD_SIZE, SUPPORT_SEAT } from "./constants";

export { NAME_MAX_CHARS, SLOT_LIMIT, SQUAD_SIZE, SUPPORT_SEAT } from "./constants";

/** The page's working copy of one squad. */
export interface ISquadDraft {
    name: string;
    /** Exactly `SQUAD_SIZE` entries. Seat 0 is the squad leader. */
    members: Array<SquadMember | null>;
    support: SquadMember | null;
    stageId: string | null;
}

export function emptyDraft(name = ""): ISquadDraft {
    return { name, members: Array.from({ length: SQUAD_SIZE }, () => null), support: null, stageId: null };
}

/** Pad or trim a seat list to exactly `SQUAD_SIZE` seats. */
export function normalizeMembers<T = SquadMember>(members: ReadonlyArray<T | null | undefined>): Array<T | null> {
    const out: Array<T | null> = [];
    for (let i = 0; i < SQUAD_SIZE; i++) out.push(members[i] ?? null);
    return out;
}

/** What sits at `seat` in a squad's seats, the support at `SUPPORT_SEAT`; `null` for an empty seat. */
export function seatAt<T>(members: ReadonlyArray<T | null>, support: T | null, seat: number): T | null {
    return seat === SUPPORT_SEAT ? support : (members[seat] ?? null);
}

export function filledSeats(draft: ISquadDraft): number {
    return draft.members.filter((m) => m !== null).length;
}

/** Deep-equal on the fields that are saved; the order of seats matters. */
export function draftsEqual(a: ISquadDraft, b: ISquadDraft): boolean {
    return JSON.stringify(normalizeDraftForCompare(a)) === JSON.stringify(normalizeDraftForCompare(b));
}

function normalizeDraftForCompare(d: ISquadDraft) {
    return { name: d.name.trim(), members: normalizeMembers(d.members), support: d.support, stageId: d.stageId };
}

// Type aliases, not interfaces: a `JsonValue[]` casts to an object type literal, which has an implicit index signature.
/** One entry of a roster row's `masteries` jsonb column. */
type IRosterMastery = { index: number; mastery: number };
/** One entry of a roster row's `modules` jsonb column. */
type IRosterModule = { id: string; level: number; locked: boolean | number };

/** One roster row as `model.ts` reads it: the view's jsonb columns typed. */
export interface IRosterBuild {
    elite: number;
    level: number;
    /** 0-based, as the game stores it. */
    potential: number;
    skill_level: number;
    default_skill: number | null;
    current_equip: string | null;
    masteries: IRosterMastery[];
    modules: IRosterModule[];
}

/** Narrow a `RosterEntry` to the fields a build needs; tolerant of a malformed jsonb column. */
export function rosterBuildOf(entry: RosterEntry): IRosterBuild {
    const masteries = Array.isArray(entry.masteries) ? (entry.masteries as IRosterMastery[]) : [];
    const modules = Array.isArray(entry.modules) ? (entry.modules as IRosterModule[]) : [];
    return {
        elite: entry.elite,
        level: entry.level,
        potential: entry.potential,
        skill_level: entry.skill_level,
        default_skill: entry.default_skill,
        current_equip: entry.current_equip,
        masteries,
        modules,
    };
}

/** The roster's unlocked module ranks for an operator, by module id; its INITIAL module (the default badge) left out. */
export function rosterModuleRanks(op: Pick<IOperatorIndexEntry, "modules">, build: IRosterBuild): Map<string, number> {
    const advanced = new Set(advancedModules(op).map((m) => m.uniEquipId));
    const out = new Map<string, number>();
    for (const m of build.modules) {
        const locked = m.locked === true || m.locked === 1;
        if (!locked && m.level > 0 && advanced.has(m.id)) out.set(m.id, m.level);
    }
    return out;
}

/** `maxLevel` of the given promotion, from the index entry's `phaseMaxLevels`. */
export function levelCap(op: Pick<IOperatorIndexEntry, "phaseMaxLevels">, phase: number): number {
    const caps = op.phaseMaxLevels;
    if (caps.length === 0) return 1;
    return caps[Math.min(Math.max(phase, 0), caps.length - 1)] ?? 1;
}

/** The highest promotion this operator can reach. */
export function maxPhase(op: Pick<IOperatorIndexEntry, "phaseMaxLevels">): number {
    return Math.max(0, op.phaseMaxLevels.length - 1);
}

type IRulesOp = Pick<IOperatorIndexEntry, "phaseMaxLevels" | "potentialRankCount" | "modules" | "skillRules">;

/** Whether a build at `phase`/`level` has reached an unlock condition. */
export function reached(u: Unlock, phase: number, level: number): boolean {
    return u.phase < phase || (u.phase === phase && u.level <= level);
}

/** The operator's top skill rank: `AllSkillLvlup` count + 1. */
export function topRank(op: Pick<IOperatorIndexEntry, "skillRules">): number {
    return op.skillRules.rankUnlocks.length + 1;
}

/** Highest skill rank the build has unlocked (`AllSkillLvlup[r - 2].UnlockCond`). */
export function maxSkillLevel(op: Pick<IOperatorIndexEntry, "skillRules">, phase: number, level: number): number {
    let rank = 1;
    for (const u of op.skillRules.rankUnlocks) {
        if (!reached(u, phase, level)) break;
        rank++;
    }
    return rank;
}

/** Masteries of skill `skillIndex` the build has unlocked (`LevelUpCostCond[m - 1].UnlockCond`); 0 below the top rank and for a skill with none. */
export function maxMastery(op: Pick<IOperatorIndexEntry, "skillRules">, skillIndex: number, phase: number, level: number, skillLevel: number): number {
    if (skillLevel < topRank(op)) return 0;
    let m = 0;
    for (const u of op.skillRules.skills[skillIndex]?.masteryUnlocks ?? []) {
        if (!reached(u, phase, level)) break;
        m++;
    }
    return m;
}

/** When skill `skillIndex` unlocks (`Skills[i].UnlockCond`); `null` for a skill the operator lacks. */
export function skillUnlock(op: Pick<IOperatorIndexEntry, "skillRules">, skillIndex: number): Unlock | null {
    return op.skillRules.skills[skillIndex]?.unlock ?? null;
}

/** Modules the index says the operator has, INITIAL excluded. */
export function advancedModules(op: Pick<IOperatorIndexEntry, "modules">): IOperatorIndexEntry["modules"] {
    return op.modules.filter((m) => m.type !== "INITIAL");
}

/** A module's type tag as the game prints it, `typeName1-typeName2` ("MAR-X"), either half left out when empty. */
export function moduleTypeLabel(m: Pick<IOperatorIndexEntry["modules"][number], "typeName1" | "typeName2">): string {
    return [m.typeName1, m.typeName2].filter(Boolean).join("-");
}

/** A module's rank count (`battle_equip_table` phases); 0 for no module. */
export function moduleRankCount(op: Pick<IOperatorIndexEntry, "modules">, moduleId: string | null): number {
    if (!moduleId) return 0;
    return op.modules.find((m) => m.uniEquipId === moduleId)?.rankCount ?? 0;
}

/** A full build: the top promotion at its cap, every potential rank, the top skill rank with every mastery and the module's top rank. */
export function fullBuild(op: IRulesOp, member: Pick<SquadMember, "skill_index" | "module_id">): SquadBuild {
    const phase = maxPhase(op);
    const level = levelCap(op, phase);
    const skill_level = maxSkillLevel(op, phase, level);
    return {
        phase,
        level,
        potential: Math.max(0, op.potentialRankCount),
        skill_level,
        mastery: maxMastery(op, member.skill_index, phase, level, skill_level),
        module_level: moduleRankCount(op, member.module_id),
    };
}

/** `value` held to `[lo, hi]`; `lo` wins when `hi` is below it. */
function clamp(value: number, lo: number, hi: number): number {
    return Math.max(lo, Math.min(hi, value));
}

/** Clamp every field of a build into what the operator, the chosen skill and the module allow at its promotion and level. */
export function clampBuild(op: IRulesOp, build: SquadBuild, member: Pick<SquadMember, "skill_index" | "module_id">): SquadBuild {
    const phase = clamp(build.phase, 0, maxPhase(op));
    const level = clamp(build.level, 1, levelCap(op, phase));
    const potential = clamp(build.potential, 0, op.potentialRankCount);
    const skill_level = clamp(build.skill_level, 1, maxSkillLevel(op, phase, level));
    const mastery = clamp(build.mastery, 0, maxMastery(op, member.skill_index, phase, level, skill_level));
    const ranks = moduleRankCount(op, member.module_id);
    const module_level = ranks > 0 ? clamp(build.module_level || 1, 1, ranks) : 0;
    return { phase, level, potential, skill_level, mastery, module_level };
}

export type BuildSource = "roster" | "hand" | "full" | "support";

export interface IResolvedSeat {
    member: SquadMember;
    build: SquadBuild;
    source: BuildSource;
}

/**
 * The build a seat is analysed at. `roster` is the signed-in user's row for
 * this operator (undefined when not owned or signed out). A support is always
 * a full build unless set by hand, the way a borrowed operator comes at the
 * lender's build.
 */
export function resolveSeat(given: SquadMember, op: IOperatorIndexEntry, roster: IRosterBuild | undefined, isSupport: boolean): IResolvedSeat {
    const lastSkill = Math.max(0, op.skillCount - 1);
    const member = given.skill_index > lastSkill ? { ...given, skill_index: lastSkill } : given;
    if (member.build) return { member, build: clampBuild(op, member.build, member), source: "hand" };
    if (isSupport || !roster) return { member, build: fullBuild(op, member), source: isSupport ? "support" : "full" };
    const ranks = rosterModuleRanks(op, roster);
    const moduleRank = member.module_id ? (ranks.get(member.module_id) ?? 0) : 0;
    const mastery = roster.masteries.find((m) => m.index === member.skill_index)?.mastery ?? 0;
    const build: SquadBuild = {
        phase: roster.elite,
        level: roster.level,
        potential: roster.potential,
        skill_level: roster.skill_level,
        mastery,
        module_level: moduleRank,
    };
    return { member, build: clampBuild(op, build, { skill_index: member.skill_index, module_id: moduleRank > 0 ? member.module_id : null }), source: "roster" };
}

/** A member as the game had it: the roster's default skill and current module, the INITIAL module (the default badge) read as none. */
export function memberFromRoster(charId: string, roster: IRosterBuild | undefined, op: Pick<IOperatorIndexEntry, "modules"> | undefined): SquadMember {
    const skill = roster?.default_skill ?? 0;
    const equip = roster?.current_equip ?? null;
    const module_id = equip && op && advancedModules(op).some((m) => m.uniEquipId === equip) ? equip : null;
    return { char_id: charId, skill_index: skill < 0 ? 0 : skill, module_id, build: null };
}

export function toMemberBuild(seat: IResolvedSeat): SquadMemberBuild {
    return {
        char_id: seat.member.char_id,
        phase: seat.build.phase,
        level: seat.build.level,
        potential: seat.build.potential,
        skill_index: seat.member.skill_index,
        skill_level: seat.build.skill_level,
        mastery: seat.build.mastery,
        module_id: seat.member.module_id,
        module_level: seat.member.module_id ? seat.build.module_level : 0,
    };
}

export interface IAnalyzeInputs {
    startDp: number;
    deployLimit: number | null;
    dub: string | null;
}

/** The `POST /squads/analyze` body for a draft whose seats are resolved. */
export function toAnalyzeRequest(members: ReadonlyArray<IResolvedSeat | null>, support: IResolvedSeat | null, inputs: IAnalyzeInputs): SquadAnalyzeRequest {
    return {
        members: normalizeMembers(members.map((s) => (s ? toMemberBuild(s) : null))),
        support: support ? toMemberBuild(support) : null,
        start_dp: inputs.startDp,
        deploy_limit: inputs.deployLimit,
        dub: inputs.dub,
    };
}

/**
 * The squad in the URL, nothing stored on our side. Search params:
 *   `s`      = `|`-joined seats, twelve of them, an empty field for an empty seat;
 *              each seat is `char_id.skill.module` or, with builds,
 *              `char_id.skill.module.phase.level.potential.skill_level.mastery.module_level`
 *              where `module` is the module id's suffix after `uniequip_` or `-` for none.
 *   `sup`    = the support seat in the same format.
 *   `n`      = the name.
 *   `st`     = the stage id.
 * Fields are `.`-separated because `char_id`s contain `_` and module ids contain `_`.
 */
export interface IShareSearch {
    s?: string;
    sup?: string;
    n?: string;
    st?: string;
}

function encodeSeat(m: SquadMember, includeBuild: boolean): string {
    const mod = m.module_id ? m.module_id.replace(/^uniequip_/, "") : "-";
    const head = `${m.char_id}.${m.skill_index}.${mod}`;
    if (!includeBuild || !m.build) return head;
    const b = m.build;
    return `${head}.${b.phase}.${b.level}.${b.potential}.${b.skill_level}.${b.mastery}.${b.module_level}`;
}

function decodeSeat(text: string): SquadMember | null {
    if (!text) return null;
    const parts = text.split(".");
    if (parts.length !== 3 && parts.length !== 9) return null;
    const [charId, skill, mod] = parts;
    if (!/^char_\w+$/.test(charId)) return null;
    // Any non-negative index decodes; resolving the seat clamps it to the operator's skills, as the backend does.
    const skill_index = Number.parseInt(skill, 10);
    if (!Number.isInteger(skill_index) || skill_index < 0) return null;
    const module_id = mod === "-" || !mod ? null : /^\w+$/.test(mod) ? `uniequip_${mod}` : null;
    if (mod !== "-" && mod && module_id === null) return null;
    let build: SquadBuild | null = null;
    if (parts.length === 9) {
        const nums = parts.slice(3).map((p) => Number.parseInt(p, 10));
        if (nums.some((n) => !Number.isInteger(n) || n < 0)) return null;
        const [phase, level, potential, skill_level, mastery, module_level] = nums;
        build = { phase, level, potential, skill_level, mastery, module_level };
    }
    return { char_id: charId, skill_index, module_id, build };
}

/**
 * Encode a draft. With `includeBuilds` every seat carries its resolved build
 * (pass `resolved` so roster-following seats are pinned to what the sender
 * saw); without it the link carries operators, skills and modules only.
 */
export function encodeShare(draft: ISquadDraft, includeBuilds: boolean, resolved?: { members: ReadonlyArray<IResolvedSeat | null>; support: IResolvedSeat | null }): IShareSearch {
    const pin = (m: SquadMember | null, r: IResolvedSeat | null | undefined): SquadMember | null => {
        if (!m) return null;
        if (!includeBuilds) return { ...m, build: null };
        return { ...m, build: r?.build ?? m.build };
    };
    const members = normalizeMembers(draft.members).map((m, i) => pin(m, resolved?.members[i]));
    const support = pin(draft.support, resolved?.support);
    const out: IShareSearch = { s: members.map((m) => (m ? encodeSeat(m, includeBuilds) : "")).join("|") };
    if (support) out.sup = encodeSeat(support, includeBuilds);
    const name = draft.name.trim();
    if (name) out.n = name.slice(0, NAME_MAX_CHARS);
    if (draft.stageId) out.st = draft.stageId;
    return out;
}

/** `null` when the params carry no squad. Malformed seats are dropped, never thrown on. */
export function decodeShare(search: IShareSearch): ISquadDraft | null {
    if (typeof search.s !== "string") return null;
    const seats = search.s.split("|").slice(0, SQUAD_SIZE).map(decodeSeat);
    const draft = emptyDraft(typeof search.n === "string" ? search.n.slice(0, NAME_MAX_CHARS) : "");
    draft.members = normalizeMembers(seats);
    draft.support = typeof search.sup === "string" ? decodeSeat(search.sup) : null;
    draft.stageId = typeof search.st === "string" && /^[\w#-]+$/.test(search.st) ? search.st : null;
    return draft;
}

export function hasShare(search: Record<string, unknown>): boolean {
    return typeof search.s === "string";
}

const SHARE_PARAMS = ["s", "sup", "n", "st"] as const satisfies ReadonlyArray<keyof IShareSearch>;

/**
 * The share params out of the router's parsed search. The router parses every
 * value as JSON first, so a squad named "123" arrives as a number: any
 * primitive is taken back as text, anything else dropped.
 */
export function parseShareSearch(search: Record<string, unknown>): IShareSearch {
    const out: IShareSearch = {};
    for (const key of SHARE_PARAMS) {
        const raw = search[key];
        if (typeof raw === "string") out[key] = raw;
        else if (typeof raw === "number" || typeof raw === "boolean") out[key] = String(raw);
    }
    return out;
}

/**
 * `?import=` on `/tools/dps`: up to four operators with their builds and the
 * flat buffs the squad gives each one, so the DPS calculator can open with
 * squadmates. Encoded as JSON in a base64url string; the route strips it
 * after reading.
 */
export interface IDpsImportSeat {
    charId: string;
    phase: number;
    level: number;
    /** 0-based. */
    potential: number;
    /** 0-based. */
    skillIndex: number;
    skillLevel: number;
    mastery: number;
    moduleId: string | null;
    moduleLevel: number;
    buffs: { atk: number; flatAtk: number; aspd: number };
}

export interface IDpsImport {
    seats: IDpsImportSeat[];
}

export function encodeDpsImport(data: IDpsImport): string {
    const json = JSON.stringify(data);
    const bytes = new TextEncoder().encode(json);
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

export function decodeDpsImport(text: string): IDpsImport | null {
    try {
        const b64 = text.replaceAll("-", "+").replaceAll("_", "/");
        const bin = atob(b64 + "=".repeat((4 - (b64.length % 4)) % 4));
        const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
        const parsed = JSON.parse(new TextDecoder().decode(bytes)) as unknown;
        if (!parsed || typeof parsed !== "object" || !Array.isArray((parsed as IDpsImport).seats)) return null;
        const seats = (parsed as IDpsImport).seats.filter((s) => s && typeof s.charId === "string").slice(0, DPS_SQUADMATE_LIMIT + 1);
        return { seats };
    } catch {
        return null;
    }
}

/**
 * What a picked stage asks of the squad. Derived client-side from
 * `stageDetailQueryOptions` (level options plus the enemy tally); the threat
 * thresholds are the game's enemy grade bands from `GET /squads/rules`.
 */
export type StageThreatKey = "high_res" | "high_def" | "aerial" | "invisible" | "multi_life" | "fast";

export interface IStageThreat {
    key: StageThreatKey;
    /** Coverage rows that answer it. Empty means shown for information only. */
    rows: CoverageRowKey[];
    /** Enemies carrying it, summed over the stage's spawns. */
    count: number;
    enemies: Array<{ enemyId: string; name: string; count: number }>;
}

export interface IStageContext {
    stageId: string;
    code: string;
    name: string;
    zoneName: string | null;
    apCost: number | null;
    deployLimit: number | null;
    startDp: number | null;
    dpPerSecond: number | null;
    lifePoints: number | null;
    enemyCount: number;
    /** `true` when the level file was present, so the numbers above are real. */
    hasLevel: boolean;
    threats: IStageThreat[];
}
