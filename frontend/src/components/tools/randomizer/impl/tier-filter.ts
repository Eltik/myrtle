import type { ITierListBrowseItem, ITierListDetail } from "#/lib/api/tier-lists";

/** Chip id for operators the chosen list does not place. Tier ids are uuids, so it cannot collide. */
export const UNRANKED_TIER_ID = "unranked";

export interface ITierChoice {
    slug: string;
    title: string;
}

export interface ITierChip {
    id: string;
    name: string;
}

/** A chosen list, reduced to what the pool filter and the chips need. */
export interface ITierIndex {
    /** The list's tiers in its own display order. */
    tiers: ITierChip[];
    /** Operator id to the id of the first tier that places it. */
    tierByOperator: Map<string, string>;
}

/** The official lists a player can filter by, in the order the browse endpoint returns them. */
export function officialTierChoices(lists: ITierListBrowseItem[] | null | undefined): ITierChoice[] {
    if (!lists) return [];
    return lists.filter((l) => l.listType === "official").map((l) => ({ slug: l.slug, title: l.title }));
}

/** The stored slug when it still names an official list; anything else reads as no tier filter. */
export function resolveTierListSlug(stored: string | null, choices: ITierChoice[]): string | null {
    if (!stored) return null;
    return choices.some((c) => c.slug === stored) ? stored : null;
}

export function buildTierIndex(detail: ITierListDetail): ITierIndex {
    const tierByOperator = new Map<string, string>();
    // `tiers` arrive sorted by display order, so a doubly placed operator keeps its higher tier.
    for (const tier of detail.tiers) {
        for (const entity of tier.entities) {
            if (entity.kind !== "operator" || tierByOperator.has(entity.id)) continue;
            tierByOperator.set(entity.id, tier.id);
        }
    }
    return { tiers: detail.tiers.map((t) => ({ id: t.id, name: t.name })), tierByOperator };
}

/** True when the operator's tier (or Unranked) is still selected. A missing index means the filter is off. */
export function passesTierFilter(operatorId: string, index: ITierIndex | null, deselectedTierIds: readonly string[]): boolean {
    if (!index || deselectedTierIds.length === 0) return true;
    const tierId = index.tierByOperator.get(operatorId) ?? UNRANKED_TIER_ID;
    return !deselectedTierIds.includes(tierId);
}

/** True when the stored deselection actually narrows the pool against this index. */
export function tierFilterNarrows(index: ITierIndex | null, deselectedTierIds: readonly string[]): boolean {
    if (!index) return false;
    return deselectedTierIds.some((id) => id === UNRANKED_TIER_ID || index.tiers.some((t) => t.id === id));
}

/** What the settings panel and roster note need to know about the tier filter. */
export interface ITierFilterState {
    /** Official lists to choose from; empty while loading or on error. */
    choices: ITierChoice[];
    /** The chosen list, already checked against `choices`. */
    slug: string | null;
    index: ITierIndex | null;
    loading: boolean;
    /** The chosen list's title when some tier is deselected, for the roster note; `null` otherwise. */
    narrowingTitle: string | null;
}
