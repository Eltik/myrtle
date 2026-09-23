import type { IRosterEntry } from "#/lib/api/user";
import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import { getActivityIdFromZoneId, getPermanentEventInfo, getPermanentZonePrefix, type IActivityLookup, isActivityCurrentlyOpen } from "./activity-lookup";
import { CHALLENGES } from "./challenges";
import { UNPLAYABLE_OPERATOR_IDS } from "./constants";
import type { IChallenge, IRandomizerOperator, IRandomizerSettings, IRosterIndex } from "./types";
import type { messages as utilMessages } from "./utils.messages";

/** The `t` the stage-grouping helper needs, narrowed to the keys it can render. */
export type RandomizerUtilsT = TypedT<typeof utilMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components render
 * and this file carries no second copy of the text.
 */
const sourceT: RandomizerUtilsT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

const HEART_OF_SURGING_FLAME_NAME = "Heart of Surging Flame";

function isHeartOfSurgingFlameStage(zoneId: string, lookup: IActivityLookup): boolean {
    const permanentPrefix = getPermanentZonePrefix(zoneId);
    if (permanentPrefix) {
        const retro = lookup.retroByZonePrefix.get(permanentPrefix);
        if (retro?.name === HEART_OF_SURGING_FLAME_NAME) return true;
    }

    const activityId = getActivityIdFromZoneId(zoneId);
    if (!activityId) return false;

    const activityName = lookup.activityById.get(activityId)?.name;
    if (activityName === HEART_OF_SURGING_FLAME_NAME || activityName === `${HEART_OF_SURGING_FLAME_NAME} - Rerun`) return true;

    const linkedRetro = lookup.retroByActivityId.get(activityId);
    if (linkedRetro?.name === HEART_OF_SURGING_FLAME_NAME) return true;

    return false;
}

/**
 * A stage that costs sanity is a battle. Heart of Surging Flame is the exception:
 * its battles cost 0 AP, so there the ST / TR codes are what mark the non-battles.
 */
function isPlayableStage(stage: IStage, lookup: IActivityLookup): boolean {
    if (stage.apCost > 0) return true;
    if (!isHeartOfSurgingFlameStage(stage.zoneId, lookup)) return false;
    const code = stage.code.toUpperCase();
    return !code.includes("ST") && !code.includes("TR");
}

/** Drops 0-AP tutorial/story-only stages (except Heart of Surging Flame playables). */
export function filterPlayableStages(stages: IStage[], lookup: IActivityLookup): IStage[] {
    return stages.filter((stage) => isPlayableStage(stage, lookup));
}

export function buildRosterIndex(roster: IRosterEntry[] | null | undefined): IRosterIndex {
    const owned = new Set<string>();
    const e2 = new Set<string>();
    if (!roster) return { owned, e2 };
    for (const entry of roster) {
        if (!entry.operator_id) continue;
        owned.add(entry.operator_id);
        if (entry.elite >= 2) e2.add(entry.operator_id);
    }
    return { owned, e2 };
}

export function toRandomizerOperator(op: IOperatorIndexEntry): IRandomizerOperator | null {
    if (!op.id) return null;
    return {
        id: op.id,
        name: op.name,
        rarity: rarityToNumber(op.rarity),
        profession: op.profession,
        subProfessionId: op.subProfessionId,
        position: op.position,
        race: op.race,
        hasOffensiveRecovery: op.hasOffensiveRecovery,
        hasDefensiveRecovery: op.hasDefensiveRecovery,
        allSkillsManual: op.allSkillsManual,
    };
}

/**
 * The settings as they apply to this session. Saved settings outlive the
 * session, but the owned / E2 / cleared filters read data only a profile
 * brings: signed out, the roster and the clears are empty, so a filter saved
 * while signed in would draw nothing ("0 drawable") behind a switch that
 * renders off and locked, where it cannot be turned off. Those filters apply
 * only with a profile, and the cleared filter only once the clears have loaded
 * (a missing clears map reads every stage as uncleared). The stored choice is
 * untouched and returns when the data does.
 */
export function applyProfileGate(settings: IRandomizerSettings, { hasProfile, hasStageClears }: { hasProfile: boolean; hasStageClears: boolean }): IRandomizerSettings {
    if (!hasProfile) return { ...settings, onlyOwnedOperators: false, onlyE2Operators: false, onlyCompletedStages: false };
    if (!hasStageClears) return { ...settings, onlyCompletedStages: false };
    return settings;
}

export function selectAvailableOperators(operators: IRandomizerOperator[], settings: IRandomizerSettings, rosterIndex: IRosterIndex): IRandomizerOperator[] {
    return operators.filter((op) => {
        if (!settings.allowedRarities.includes(op.rarity)) return false;
        if (!settings.allowedClasses.includes(op.profession)) return false;
        if (settings.hideUnplayableOperators && UNPLAYABLE_OPERATOR_IDS.has(op.id)) return false;
        if (settings.onlyOwnedOperators && !rosterIndex.owned.has(op.id)) return false;
        if (settings.onlyE2Operators && !rosterIndex.e2.has(op.id)) return false;
        return true;
    });
}

/** Zone types whose stages are always available (mainline, retro, daily farming, annihilation). */
const PERMANENT_ZONE_TYPES = new Set(["MAINLINE", "MAINLINE_ACTIVITY", "MAINLINE_RETRO", "WEEKLY", "CAMPAIGN"]);

/** Playable right now: permanent content, or an event whose window is open at `now` (unix seconds). */
function isStageAvailableNow(stage: IStage, zone: IZone, lookup: IActivityLookup, now: number): boolean {
    if (PERMANENT_ZONE_TYPES.has(zone.type)) return true;
    if (getPermanentZonePrefix(stage.zoneId)) return true;
    const activityId = getActivityIdFromZoneId(stage.zoneId);
    if (!activityId) return false;
    if (getPermanentEventInfo(activityId, lookup)) return true;
    return isActivityCurrentlyOpen(activityId, lookup, now);
}

export function selectAvailableStages(stages: IStage[], zones: IZone[], settings: IRandomizerSettings, stageClears: StageClearsMap | null | undefined, lookup: IActivityLookup): IStage[] {
    const zoneById = new Map(zones.map((z) => [z.zoneId, z]));
    const deselectedStages = new Set(settings.deselectedStageIds);
    const now = Math.floor(Date.now() / 1000);

    return stages.filter((stage) => {
        const zone = zoneById.get(stage.zoneId);
        if (!zone) return false;
        if (!settings.allowedZoneTypes.includes(zone.type)) return false;
        if (deselectedStages.has(stage.stageId)) return false;
        if (settings.onlyCompletedStages && !isStageCleared(stage.stageId, stageClears)) return false;
        if (settings.onlyAvailableStages) return isStageAvailableNow(stage, zone, lookup, now);
        return true;
    });
}

export function pickRandomStage(stages: IStage[]): IStage | null {
    if (stages.length === 0) return null;
    return stages[Math.floor(Math.random() * stages.length)] ?? null;
}

export function pickRandomSquad(operators: IRandomizerOperator[], squadSize: number, allowDuplicates: boolean): IRandomizerOperator[] {
    if (operators.length === 0) return [];

    const out: IRandomizerOperator[] = [];
    const pool = [...operators];

    for (let i = 0; i < squadSize; i++) {
        if (pool.length === 0) {
            if (allowDuplicates) {
                pool.push(...operators);
            } else {
                break;
            }
        }
        const idx = Math.floor(Math.random() * pool.length);
        const chosen = pool[idx];
        if (!chosen) break;
        out.push(chosen);
        if (!allowDuplicates) pool.splice(idx, 1);
    }

    return out;
}

export interface IChallengePickContext {
    /** The rolled stage. Stage-specific challenges check this. */
    stage: IStage;
    /**
     * Operator pool after settings + roster filtering. Squad-filter challenges
     * run their predicate against this list.
     */
    operators: IRandomizerOperator[];
    /** Minimum pool size a SQUAD_FILTER challenge must yield to be eligible. */
    squadSize: number;
}

export interface IPickedChallenge {
    challenge: IChallenge;
    /** For SQUAD_FILTER: the operators that survived the filter. Undefined for PLAIN/STAGE. */
    filteredOperators?: IRandomizerOperator[];
}

/**
 * Weighted random pick over the challenge registry, respecting eligibility:
 *   - PLAIN: always eligible.
 *   - STAGE: eligible iff match(stage) returns true.
 *   - SQUAD_FILTER: eligible iff the filtered pool has at least squadSize operators.
 *
 * Returns null only if no challenge is eligible (e.g. empty operator pool and no
 * applicable stage / plain challenges in the registry).
 */
export function pickRandomChallenge(ctx: IChallengePickContext): IPickedChallenge | null {
    type Entry = { challenge: IChallenge; weight: number; filteredOperators?: IRandomizerOperator[] };
    const eligible: Entry[] = [];

    for (const ch of CHALLENGES) {
        const weight = ch.weight ?? 1;
        if (weight <= 0) continue;

        if (ch.type === "PLAIN") {
            eligible.push({ challenge: ch, weight });
        } else if (ch.type === "STAGE") {
            if (ch.match(ctx.stage)) eligible.push({ challenge: ch, weight });
        } else {
            const filtered = ctx.operators.filter(ch.filter);
            if (filtered.length >= ctx.squadSize) {
                eligible.push({ challenge: ch, weight, filteredOperators: filtered });
            }
        }
    }

    if (eligible.length === 0) return null;

    const total = eligible.reduce((s, e) => s + e.weight, 0);
    let r = Math.random() * total;
    for (const e of eligible) {
        r -= e.weight;
        if (r <= 0) return { challenge: e.challenge, filteredOperators: e.filteredOperators };
    }
    const last = eligible[eligible.length - 1];
    return last ? { challenge: last.challenge, filteredOperators: last.filteredOperators } : null;
}

export function getZoneDisplayName(zone: IZone | undefined, stageZoneId: string): string {
    if (!zone) return stageZoneId;
    return zone.zoneNameSecond ?? zone.zoneNameFirst ?? stageZoneId;
}

/**
 * A stage is cleared when its `state` is 3 (passed). Note that for adverse/tough
 * variants, clearing the harder version auto-passes the Normal/Easy variant with
 * `state: 3` and `completeTimes: 0`, so checking `completeTimes` alone would miss them.
 */
export function isStageCleared(stageId: string, stageClears: StageClearsMap | null | undefined): boolean {
    const clear = stageClears?.[stageId];
    return !!clear && (clear.state ?? 0) >= 3;
}

export type StageGroupSection = "MAIN" | "EVENT" | "OTHER";

/** Display order of the sections; the same order `buildStageGroups` sorts by. */
export const STAGE_SECTION_ORDER: readonly StageGroupSection[] = ["MAIN", "EVENT", "OTHER"];

export interface IStageGroup {
    id: string;
    label: string;
    sublabel?: string;
    section: StageGroupSection;
    /** Underlying zone type bucket (MAINLINE vs ACTIVITY) for cross-referencing source filters. */
    zoneType: string;
    /** Whether the event is currently open / always-available (permanent or mainline). */
    isOpen: boolean;
    sortKey: number;
    stages: IStage[];
}

/** Everything a group takes from the first stage that lands in it. */
type StageGroupHead = Pick<IStageGroup, "id" | "label" | "sublabel" | "section" | "isOpen" | "sortKey">;

function natCompareCode(a: string, b: string): number {
    return a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" });
}

/** Activity ID suffixes that mark non-mainstream content (mini events, sandbox, fun, etc.). */
const OTHER_ACTIVITY_SUFFIXES = ["mini", "fun", "festival", "melee", "rune", "sandbox", "april", "vsint", "trial", "training"];

function isOtherActivity(activityId: string): boolean {
    const m = activityId.match(/^act\d+([a-z]+)$/i);
    if (!m) return false;
    const suffix = (m[1] ?? "").toLowerCase();
    return OTHER_ACTIVITY_SUFFIXES.includes(suffix);
}

/**
 * Extract the activity id from a stageId prefix. Some legacy event stages
 * (e.g. `act7d5_04`, `act4d0_05`) are attached to a `main_X` zone in the
 * game data despite being event content; the stageId itself is the reliable
 * source of truth for which activity they actually belong to.
 */
function getActivityIdFromStageId(stageId: string): string | null {
    const underscore = stageId.indexOf("_");
    if (underscore <= 0) return null;
    const prefix = stageId.slice(0, underscore);
    return /^act\d/i.test(prefix) ? prefix : null;
}

const MAINLINE_ZONE_TYPES = new Set(["MAINLINE", "MAINLINE_ACTIVITY", "MAINLINE_RETRO"]);

/**
 * Genuine main story: a mainline zone AND a `main_/tough_/easy_/hard_/st_` stage
 * id. Legacy event stages (e.g. `act7d5_04`) sometimes share a `main_X` zone
 * but are not main story.
 */
function isMainStoryStage(stage: IStage, zone: IZone): boolean {
    const stageIdPrefix = stage.stageId.split("_", 1)[0] ?? "";
    return MAINLINE_ZONE_TYPES.has(zone.type) && /^(main|tough|easy|hard|st)$/i.test(stageIdPrefix);
}

function mainStoryGroupHead(zone: IZone, t: RandomizerUtilsT): StageGroupHead {
    const chapterNumber = zone.zoneNameTitleCurrent?.replace(/^0+/, "") || String(zone.zoneIndex ?? 0);
    const chapterName = zone.zoneNameSecond ?? zone.zoneNameFirst ?? zone.zoneId;
    const chapterSortKey = Number.parseInt(zone.zoneNameTitleCurrent ?? "", 10);
    return {
        // Keyed by chapter number so the MAINLINE / MAINLINE_ACTIVITY /
        // MAINLINE_RETRO variants of one chapter merge into one entry.
        id: `mainline:${chapterNumber || zone.zoneId}`,
        label: t("randomizer.stages.chapter", { number: chapterNumber, name: chapterName }),
        sublabel: zone.zoneNameFirst && zone.zoneNameFirst !== chapterName ? zone.zoneNameFirst : undefined,
        section: "MAIN",
        isOpen: true,
        sortKey: Number.isFinite(chapterSortKey) ? chapterSortKey : (zone.zoneIndex ?? 0),
    };
}

/** An event group. A retro'd side story or branchline is permanent; anything else is open only inside its window. */
function activityGroupHead(activityId: string, fallbackLabel: string, lookup: IActivityLookup, now: number, t: RandomizerUtilsT): StageGroupHead {
    const activity = lookup.activityById.get(activityId);
    const retro = lookup.retroByActivityId.get(activityId);
    const isPermanentSideOrBranch = !!retro && (retro.type === "SIDESTORY" || retro.type === "BRANCHLINE");

    let sublabel: string | undefined;
    let isOpen = false;
    if (isPermanentSideOrBranch) {
        sublabel = t("randomizer.stages.permanent");
        isOpen = true;
    } else if (activity) {
        isOpen = activity.startTime <= now && now <= activity.endTime;
        sublabel = activity.isReplicate ? t("randomizer.stages.rerun") : undefined;
    }

    return {
        id: `activity:${activityId}`,
        label: activity?.name ?? retro?.name ?? fallbackLabel,
        sublabel,
        section: isOtherActivity(activityId) && !isPermanentSideOrBranch ? "OTHER" : "EVENT",
        isOpen,
        sortKey: activity?.startTime ?? retro?.startTime ?? 0,
    };
}

/**
 * Which group a stage belongs to. The stage id's own activity wins over the
 * zone's (see `getActivityIdFromStageId`); then a permanent retro zone, the
 * zone's activity, and finally the bare zone.
 */
function stageGroupHead(stage: IStage, zone: IZone, lookup: IActivityLookup, now: number, t: RandomizerUtilsT): StageGroupHead {
    if (isMainStoryStage(stage, zone)) return mainStoryGroupHead(zone, t);

    const stageActivityId = getActivityIdFromStageId(stage.stageId);
    if (stageActivityId && (lookup.activityById.has(stageActivityId) || lookup.retroByActivityId.has(stageActivityId))) {
        return activityGroupHead(stageActivityId, stageActivityId, lookup, now, t);
    }

    const zoneLabel = getZoneDisplayName(zone, zone.zoneId);
    const permanentPrefix = getPermanentZonePrefix(stage.zoneId);
    if (permanentPrefix) {
        const retro = lookup.retroByZonePrefix.get(permanentPrefix);
        return { id: `permanent:${permanentPrefix}`, label: retro?.name ?? zoneLabel, sublabel: t("randomizer.stages.permanent"), section: "EVENT", isOpen: true, sortKey: retro?.startTime ?? 0 };
    }

    const zoneActivityId = getActivityIdFromZoneId(stage.zoneId);
    if (zoneActivityId) return activityGroupHead(zoneActivityId, zoneLabel, lookup, now, t);

    return { id: `zone:${stage.zoneId}`, label: zoneLabel, sublabel: undefined, section: "OTHER", isOpen: PERMANENT_ZONE_TYPES.has(zone.type), sortKey: zone.zoneIndex ?? 0 };
}

/**
 * Bucket the supplied stages into user-facing groups partitioned into three sections:
 * Main Story (one per mainline episode), Events (full sidestories / permanent SS/BL),
 * and Other (mini events, sandbox, festivals, etc.). Main Story runs oldest chapter
 * first; the other sections newest first.
 */
export function buildStageGroups(stages: IStage[], zones: IZone[], lookup: IActivityLookup, t: RandomizerUtilsT = sourceT): IStageGroup[] {
    const zoneById = new Map(zones.map((z) => [z.zoneId, z]));
    const now = Math.floor(Date.now() / 1000);
    const groups = new Map<string, IStageGroup>();

    for (const stage of stages) {
        const zone = zoneById.get(stage.zoneId);
        if (!zone) continue;
        const head = stageGroupHead(stage, zone, lookup, now, t);
        let group = groups.get(head.id);
        if (!group) {
            group = { ...head, zoneType: zone.type, stages: [] };
            groups.set(head.id, group);
        }
        group.stages.push(stage);
    }

    const out = Array.from(groups.values());
    for (const g of out) {
        g.stages.sort((a, b) => natCompareCode(a.code, b.code) || a.stageId.localeCompare(b.stageId));
    }
    out.sort((a, b) => {
        const sec = STAGE_SECTION_ORDER.indexOf(a.section) - STAGE_SECTION_ORDER.indexOf(b.section);
        if (sec !== 0) return sec;
        if (a.section === "MAIN") return a.sortKey - b.sortKey;
        return b.sortKey - a.sortKey;
    });
    return out;
}

export const STAGE_SECTION_LABEL_KEYS: Record<StageGroupSection, keyof typeof utilMessages & string> = {
    MAIN: "randomizer.section.main",
    EVENT: "randomizer.section.event",
    OTHER: "randomizer.section.other",
};
