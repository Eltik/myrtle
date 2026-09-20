import type { FilterSets } from "#/components/operators/list/impl/shared-filters";
import { hasAnySharedFilter, matchesSharedFilters } from "#/components/operators/list/impl/shared-filters";
import { compactForSearch } from "#/lib/search/fuzzy";
import { isMaxed, MAX_ELITE_BY_RARITY, MAX_LEVEL_BY_RARITY } from "./helpers.card";
import type { IDisplayEntry, SortKey, SortOrder, SourceFilter } from "./types";

/**
 * Classifies `character_table.ItemObtainApproach`. On EN (2026-09-21) every
 * gacha operator's approach contains "Headhunting": 313 read "Recruitment &
 * Headhunting" and Texas alone reads "Recruitment & Headhunting, Pinboard
 * Missions", which is why this is a substring test and not an equality. The
 * other side is "Event Reward" (70), "Voucher Exchange" (12), "Obtained from
 * Integrated Strategies" (5), "Credit Store" (3), "Anniversary Reward"
 * (Savage), "Main Theme Story" (Amiya), "Limited Gift Pack" (Purestream).
 *
 * `null` is an operator with NO approach: the backend serialises the missing
 * field as "" for the 29 `isNotObtainable` rows (Reserve Operators, IS-only
 * temporaries), and an entry whose static row has not loaded looks the same.
 * Neither side of the filter claims those, so "any" is the only view that
 * shows them.
 */
export function operatorSource(approach: string | null | undefined): Exclude<SourceFilter, "any"> | null {
    if (!approach) return null;
    return approach.includes("Headhunting") ? "headhunting" : "welfare";
}

export function filterEntries(entries: IDisplayEntry[], search: string, sets: FilterSets, source: SourceFilter = "any"): IDisplayEntry[] {
    const q = compactForSearch(search);
    const active = hasAnySharedFilter(sets);
    if (!q && !active && source === "any") return entries;

    return entries.filter((e) => {
        if (q && !compactForSearch(e.name).includes(q)) return false;
        if (source !== "any" && operatorSource(e.static?.itemObtainApproach) !== source) return false;
        if (!active) return true;
        // An operator missing from the index cannot be shown as matching any attribute filter.
        if (!e.meta) return false;
        return matchesSharedFilters({ ...e.meta, voiceActors: e.voiceActors }, sets);
    });
}

function levelKey(e: IDisplayEntry): number {
    return e.isOwned ? e.elite * 1000 + e.level : -1;
}

function obtainedKey(e: IDisplayEntry): number {
    return e.isOwned ? (e.obtained_at ?? 0) : -1;
}

function potentialKey(e: IDisplayEntry): number {
    return e.isOwned ? e.potential : -1;
}

function trustKey(e: IDisplayEntry): number {
    return e.isOwned ? e.favor_point : -1;
}

function maxedKey(e: IDisplayEntry): number {
    return e.isOwned && isMaxed(e) ? 1 : 0;
}

/**
 * How far an operator is along its OWN ceiling, in [0, 1].
 *
 * The point of this is comparability across rarities: rarity already sorts by
 * rarity, so an investment score that rewarded raw level would just be a
 * blurrier rarity sort. A 5-star at E2 90 with M3s and a module scores 1.000
 * and outranks an E0 1 6-star at 0.000, and that is the ordering the roster was
 * asked for.
 *
 * A term that CANNOT apply to this operator has its weight redistributed across
 * the terms that can, rather than being scored as complete. The difference
 * matters: scoring the inapplicable as 1 made the function NON-MONOTONIC,
 * because masteries are inapplicable below E2 and merely empty at E2, so
 * promoting a 6-star from E1 80 S7 to E2 1 S7 dropped its score 0.707 -> 0.697.
 * Investment went up and the number went down. Redistribution makes the same
 * step 0.502 -> 0.667, drops a fully-uninvested 6-star from 0.250 to 0.000, and
 * still lets a finished 3-star or 1-star, which can never have masteries or
 * modules, reach a full 1.000. Measured, and pinned by helpers.test.ts.
 *
 * Weights are a TRADE, not a derivation: promotion and level carry the bulk
 * because they gate everything else, masteries and modules carry the long tail
 * because they are where a finished operator's remaining cost sits, and
 * potential carries the least because it is not a spend the player controls.
 * They sum to 1. An unowned entry scores -1 so it sorts below every owned one
 * in descending order, matching the other owned-only keys above.
 */
const W_ELITE = 0.3;
const W_LEVEL = 0.28;
const W_SKILL = 0.12;
const W_MASTERY = 0.16;
const W_MODULE = 0.09;
const W_POTENTIAL = 0.05;

export function investmentScore(e: IDisplayEntry): number {
    if (!e.isOwned) return -1;

    const maxElite = MAX_ELITE_BY_RARITY[e.rarity] ?? 2;
    const maxLevel = MAX_LEVEL_BY_RARITY[e.rarity]?.[e.elite] ?? 90;

    // Each term is [weight, progress] or null when it cannot apply here.
    const terms: Array<[number, number] | null> = [];

    // Level is scored WITHIN the current promotion, then folded in behind it:
    // E1 1 is further along than E0 max, which is what "investment" means to a
    // player even though the raw level number went down.
    const level = maxLevel > 1 ? Math.min(Math.max(e.level - 1, 0), maxLevel - 1) / (maxLevel - 1) : 1;

    if (maxElite > 0) {
        terms.push([W_ELITE, Math.min(e.elite, maxElite) / maxElite]);
        terms.push([W_LEVEL, (Math.min(e.elite, maxElite) + level) / (maxElite + 1)]);
    } else {
        // 1-star and 2-star operators do not promote; level is the whole story.
        terms.push([W_LEVEL, level]);
    }

    // 1-star and 2-star operators have no skills.
    terms.push(e.rarity > 2 ? [W_SKILL, Math.min(Math.max(e.skill_level - 1, 0), 6) / 6] : null);

    // Applicability is a fact about the OPERATOR, not its current promotion: a
    // 6-star can reach masteries, so the term applies from E0 and simply reads
    // zero until they are earned. Only a rarity that can never master is
    // excluded.
    const canMaster = maxElite === 2 && e.rarity >= 4;
    terms.push(canMaster ? [W_MASTERY, e.masteries.length > 0 ? e.masteries.reduce((sum, m) => sum + Math.min(Math.max(m.mastery, 0), 3), 0) / (e.masteries.length * 3) : 0] : null);

    // Likewise: an operator with no module released has no module term at all,
    // while one that has modules and has unlocked none reads zero.
    terms.push(e.modules.length > 0 ? [W_MODULE, e.modules.reduce((sum, m) => sum + Math.min(Math.max(m.level, 0), 3), 0) / (e.modules.length * 3)] : null);

    // Potential is 0-indexed in the roster row: `isMaxed` treats 5 as full.
    terms.push([W_POTENTIAL, Math.min(Math.max(e.potential, 0), 5) / 5]);

    let weight = 0;
    let earned = 0;
    for (const term of terms) {
        if (!term) continue;
        weight += term[0];
        earned += term[0] * term[1];
    }
    return weight > 0 ? earned / weight : 0;
}

function cmpByKey(a: IDisplayEntry, b: IDisplayEntry, key: SortKey): number {
    switch (key) {
        case "investment":
            return investmentScore(a) - investmentScore(b);
        case "rarity":
            return a.rarity - b.rarity;
        case "level":
            return levelKey(a) - levelKey(b);
        case "obtained":
            return obtainedKey(a) - obtainedKey(b);
        case "potential":
            return potentialKey(a) - potentialKey(b);
        case "trust":
            return trustKey(a) - trustKey(b);
        case "maxed":
            return maxedKey(a) - maxedKey(b);
    }
}

/**
 * Ties break by investment, then rarity, then name.
 *
 * This used to break straight to `name.localeCompare`, which put an alphabetic
 * accident between two operators the player thinks of as very differently
 * invested: inside one rarity band, a maxed operator and an E0 1 operator sat
 * wherever their names fell. Each key's own secondary term is dropped from
 * `cmpByKey` for the same reason - the chain below is now the single place tie
 * order is decided, so every key gets the same one.
 */
export function sortEntries(entries: IDisplayEntry[], key: SortKey, order: SortOrder): IDisplayEntry[] {
    const dir = order === "asc" ? 1 : -1;
    // Investment is the dominant tie-break, so it is computed once per entry
    // rather than once per comparison: the comparator runs O(n log n) times.
    const investment = new Map<IDisplayEntry, number>();
    for (const e of entries) investment.set(e, investmentScore(e));

    return [...entries].sort((a, b) => {
        const primary = cmpByKey(a, b, key) * dir;
        if (primary !== 0) return primary;

        // The tie-break chain keeps the direction of the primary sort, so a
        // descending sort reads "most invested first" all the way down.
        if (key !== "investment") {
            const inv = ((investment.get(a) ?? 0) - (investment.get(b) ?? 0)) * dir;
            if (inv !== 0) return inv;
        }
        if (key !== "rarity") {
            const rarity = (a.rarity - b.rarity) * dir;
            if (rarity !== 0) return rarity;
        }
        return a.name.localeCompare(b.name);
    });
}
