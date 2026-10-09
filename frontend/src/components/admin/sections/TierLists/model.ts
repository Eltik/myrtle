/**
 * The Tier lists section's pure model: which lists a user sees, at what level,
 * how the toolbar filters them, and what each level allows. No React, no
 * session, so the tests import it directly.
 */
import { atLeast } from "#/components/admin/shell/model";
import type { AdminTierListsFilter } from "#/components/admin/shell/search";
import type { TierListPermissionLevel } from "#/lib/api/admin/types";
import type { TierListType } from "#/lib/api/tier-lists";
import { normalizeForSearch } from "#/lib/search/fuzzy";

/** Which list types a staff member is looking at. Non-staff always see every list they were granted. */
export const LIST_TYPE_SCOPES = ["official", "community", "all"] as const;
export type ListTypeScope = (typeof LIST_TYPE_SCOPES)[number];

export type ListType = TierListType;

/** The slice of a browse item the table reads. */
export interface IBrowseListLike {
    slug: string;
    title: string;
    listType: ListType;
    author: { name: string };
    flairCode: string | null;
    flairLabel: string | null;
    flairColor: string | null;
    isTrending: boolean;
    tiers: readonly { operators: readonly unknown[] }[];
    views24h: number;
    updatedAtMs: number;
}

/** The slice of a `/tier-lists/granted` row the table reads. */
export interface IGrantedListLike {
    slug: string;
    title: string;
    listType: ListType;
    permission: TierListPermissionLevel;
}

export interface ITierListRow {
    slug: string;
    title: string;
    listType: ListType;
    /** `null` when the list is not in the public browse feed (unlisted): no author or stats to show. */
    author: string | null;
    flairCode: string | null;
    flairLabel: string | null;
    flairColor: string | null;
    hot: boolean;
    tiers: number | null;
    placements: number | null;
    views24h: number | null;
    updatedAtMs: number | null;
    level: TierListPermissionLevel;
}

function fromBrowse(item: IBrowseListLike, level: TierListPermissionLevel): ITierListRow {
    return {
        slug: item.slug,
        title: item.title,
        listType: item.listType,
        author: item.author.name,
        flairCode: item.flairCode,
        flairLabel: item.flairLabel,
        flairColor: item.flairColor,
        hot: item.isTrending,
        tiers: item.tiers.length,
        placements: item.tiers.reduce((n, tier) => n + tier.operators.length, 0),
        views24h: item.views24h,
        updatedAtMs: item.updatedAtMs || null,
        level,
    };
}

/** Staff: every browse list of the chosen type, at `admin`. */
export function staffRows(browse: readonly IBrowseListLike[], scope: ListTypeScope): ITierListRow[] {
    return browse.filter((l) => scope === "all" || l.listType === scope).map((l) => fromBrowse(l, "admin"));
}

/**
 * Non-staff: one row per granted list, at the granted level. Stats come from
 * the browse feed when the list is in it; an unlisted list still gets a row,
 * with its title from the grant and no stats.
 */
export function grantedRows(browse: readonly IBrowseListLike[], grants: readonly IGrantedListLike[]): ITierListRow[] {
    const bySlug = new Map(browse.map((l) => [l.slug, l]));
    return grants.map((g) => {
        const item = bySlug.get(g.slug);
        if (item) return fromBrowse(item, g.permission);
        return { slug: g.slug, title: g.title, listType: g.listType, author: null, flairCode: null, flairLabel: null, flairColor: null, hot: false, tiers: null, placements: null, views24h: null, updatedAtMs: null, level: g.permission };
    });
}

/** Matches title, slug or flair label, accent- and case-insensitively. */
export function matchesQuery(row: ITierListRow, q: string): boolean {
    const needle = normalizeForSearch(q.trim());
    if (!needle) return true;
    return [row.title, row.slug, row.flairLabel ?? ""].some((field) => normalizeForSearch(field).includes(needle));
}

export function filterRows(rows: readonly ITierListRow[], filter: AdminTierListsFilter, q: string): ITierListRow[] {
    return rows.filter((r) => (filter === "all" || r.hot) && matchesQuery(r, q));
}

export function filterCounts(rows: readonly ITierListRow[]): { all: number; hot: number } {
    return { all: rows.length, hot: rows.filter((r) => r.hot).length };
}

/** Open the editor: `edit` or better. */
export function canEditList(level: TierListPermissionLevel): boolean {
    return atLeast(level, "edit");
}

/** Publish a version: `publish` or better (backend `Permission::Publish`). */
export function canPublishList(level: TierListPermissionLevel): boolean {
    return atLeast(level, "publish");
}

/** Change the flair: `edit` or better (backend `Permission::Edit`). */
export function canSetFlair(level: TierListPermissionLevel): boolean {
    return atLeast(level, "edit");
}

/** Delete the list: `admin` only (backend `Permission::Admin`). */
export function canDeleteList(level: TierListPermissionLevel): boolean {
    return atLeast(level, "admin");
}

/** The number the next publish will get: one past the highest published version. */
export function nextVersionNumber(versions: readonly { version: number }[]): number {
    return versions.reduce((max, v) => Math.max(max, v.version), 0) + 1;
}

/**
 * Changing a grant's level. Grants are rows keyed by (list, user, level), and
 * a grant POST never replaces an existing row, so a change is a grant of the
 * new level followed by a revoke of the old one. `null` = nothing to do.
 */
export function levelChangePlan(from: TierListPermissionLevel, to: TierListPermissionLevel): { grant: TierListPermissionLevel; revoke: TierListPermissionLevel } | null {
    return from === to ? null : { grant: to, revoke: from };
}
