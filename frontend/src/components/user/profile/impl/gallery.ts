/**
 * Pure derivations for the background editor's gallery: its three sources (the Archives
 * gallery, the story CGs, the story scene plates) flattened to one tile shape,
 * the category and story filters each source offers with their counts, and which tiles
 * a filter and a search leave.
 */

import { searchAndRank } from "#/lib/search/fuzzy";
import type { GalleryPicture } from "#/types/generated/GalleryPicture";
import type { StoryArtGallery } from "#/types/generated/StoryArtGallery";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { StoryGallery } from "#/types/generated/StoryGallery";
import type { GalleryKind } from "./background";

/** One tile: a picture of one source and the story or archive group it is filed under. */
export interface IGalleryTile {
    kind: GalleryKind;
    id: string;
    /** The picture's own title; a story CG or scene has none and prints its story's name. */
    title: string;
    groupId: string;
    groupName: string;
    /** The library category the tile files under (Main Story, Side Stories, ...); see {@link archiveCategory} for an Archives picture. */
    category: StoryCategory;
    /** The Archives picture, for its description; absent on a story picture. */
    picture?: GalleryPicture;
}

/**
 * The library category of an Archives group. The archive lists two shapes of group: an
 * Integrated Strategies theme (`rogue_1`..`rogue_5`) and an event's archive (`act13side`,
 * `act17side`, `act25side`, all three SIDESTORY events). On EN 2026-10-06 those are all 8
 * groups (5 themes, 3 events, 324 pictures), so a theme files under `is` and anything else
 * under `side`. A new shape would land in `side` until named here.
 */
export function archiveCategory(groupId: string): StoryCategory {
    return /^rogue_\d+$/.test(groupId) ? "is" : "side";
}

/** The Archives gallery's tiles, in the archive's own order. */
export function archiveTiles(gallery: StoryGallery): IGalleryTile[] {
    return gallery.groups.flatMap((group) => group.pictures.map((picture) => ({ kind: "archive_pic" as const, id: picture.id, title: picture.title, groupId: group.id, groupName: group.name, category: archiveCategory(group.id), picture })));
}

/** A story CG or scene catalogue's tiles, in library order. */
export function storyArtTiles(gallery: StoryArtGallery): IGalleryTile[] {
    const kind = gallery.kind === "cg" ? "story_cg" : "story_scene";
    return gallery.groups.flatMap((group) => group.pictures.map((picture) => ({ kind, id: picture.id, title: "", groupId: group.id, groupName: group.name, category: group.category })));
}

/** The library categories in the Stories tab's order: the category row's order. */
const STORY_CATEGORIES = ["main", "side", "vignette", "is", "reclamation", "sideContent", "record"] as const satisfies readonly StoryCategory[];

/** What narrows a source's tiles. Every part combines with the others (AND); an empty part filters nothing. */
export interface IGalleryFilter {
    /** Any of these categories (OR within the row), none chosen for every category. */
    categories: readonly StoryCategory[];
    /** One story or archive group, `null` for every group. */
    group: string | null;
    query: string;
}

export const NO_GALLERY_FILTER: IGalleryFilter = { categories: [], group: null, query: "" };

/**
 * `filter` read against one source: a category the source has no tile in, or a group it
 * does not hold, is dropped. The author's choice is kept in state, so going back to the
 * source that has it filters as it was left, and a source never opens empty because of a
 * filter set on another (the Archives have no Main Story; a CG's story may draw no scene).
 */
export function filterForSource(tiles: readonly IGalleryTile[], filter: IGalleryFilter): IGalleryFilter {
    const categories = filter.categories.filter((category) => tiles.some((tile) => tile.category === category));
    const group = filter.group !== null && tiles.some((tile) => tile.groupId === filter.group) ? filter.group : null;
    if (categories.length === filter.categories.length && group === filter.group) return filter;
    return { ...filter, categories, group };
}

/** Whether a tile passes the category and group parts of a filter. */
function passes(tile: IGalleryTile, filter: IGalleryFilter): boolean {
    return (filter.categories.length === 0 || filter.categories.includes(tile.category)) && (filter.group === null || tile.groupId === filter.group);
}

/**
 * The tiles a filter leaves. With no query they keep the source's own order; with one they
 * are ranked by `searchAndRank` over the title (the story's name where the picture has
 * none), the group's name and the id riding as `extra`, so "mizuki" finds that theme's
 * pictures and "chernobog" a chapter's CGs.
 */
export function filterGallery(tiles: readonly IGalleryTile[], filter: IGalleryFilter): IGalleryTile[] {
    const kept = tiles.filter((tile) => passes(tile, filter));
    if (filter.query.trim().length === 0) return kept;
    return searchAndRank(filter.query, kept, (tile) => ({ name: tile.title || tile.groupName, extra: `${tile.groupName} ${tile.id}` })).map((scored) => scored.item);
}

/** One category toggle: how many tiles choosing it (alone) would leave under the rest of the filter. */
export interface ICategoryOption {
    id: StoryCategory;
    count: number;
}

/**
 * The category row of a source: every category the source has a tile in, in
 * {@link STORY_CATEGORIES} order, each counted under the group and the query but NOT the
 * category selection itself (the usual facet count: what the toggle would add), so a
 * count of 0 says the current story or search has nothing there.
 */
export function categoryOptions(tiles: readonly IGalleryTile[], filter: IGalleryFilter): ICategoryOption[] {
    const present = new Set(tiles.map((tile) => tile.category));
    const counted = filterGallery(tiles, { ...filter, categories: [] });
    return STORY_CATEGORIES.filter((id) => present.has(id)).map((id) => ({ id, count: counted.filter((tile) => tile.category === id).length }));
}

/** One story or archive group in the story list. */
export interface IGroupOption {
    id: string;
    name: string;
    category: StoryCategory;
    count: number;
}

/**
 * The story list of a source, in its own order, each group counted under the categories
 * and the query but NOT the group selection. A group the categories or the query leave
 * empty is dropped, except the chosen one, which stays listed (at 0) so it can be seen and
 * cleared.
 */
export function groupOptions(tiles: readonly IGalleryTile[], filter: IGalleryFilter): IGroupOption[] {
    const counts = new Map<string, number>();
    for (const tile of filterGallery(tiles, { ...filter, group: null })) counts.set(tile.groupId, (counts.get(tile.groupId) ?? 0) + 1);
    const out = new Map<string, IGroupOption>();
    for (const tile of tiles) {
        if (out.has(tile.groupId)) continue;
        const count = counts.get(tile.groupId) ?? 0;
        if (count > 0 || tile.groupId === filter.group) out.set(tile.groupId, { id: tile.groupId, name: tile.groupName, category: tile.category, count });
    }
    return [...out.values()];
}

/** The story list narrowed by its own search box, through `fuzzy.ts`; a blank search keeps the list's order. */
export function searchGroups(groups: readonly IGroupOption[], query: string): IGroupOption[] {
    if (query.trim().length === 0) return [...groups];
    return searchAndRank(query, groups, (group) => ({ name: group.name, extra: group.id })).map((scored) => scored.item);
}

/** Whether any part of a filter narrows the tiles. */
export function anyGalleryFilter(filter: IGalleryFilter): boolean {
    return filter.categories.length > 0 || filter.group !== null || filter.query.trim().length > 0;
}

/** How many tiles the grid mounts at first, and adds each time its end scrolls into view. */
export const TILE_PAGE = 120;

/** The pages to mount so the tile at `index` is among them: one page at least, and one when it is not in the list (`-1`). */
export function pagesFor(index: number): number {
    return index < 0 ? 1 : Math.floor(index / TILE_PAGE) + 1;
}

/** The tiles to mount for `pages` pages, never fewer than one page. */
export function visibleTiles<T>(tiles: readonly T[], pages: number): readonly T[] {
    return tiles.slice(0, TILE_PAGE * Math.max(1, Math.floor(pages)));
}
