import { RotateCwIcon } from "lucide-react";
import { type ReactNode, useMemo } from "react";
import { Button } from "#/components/ui/button";
import { Spinner } from "#/components/ui/spinner";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { GalleryKind } from "../../background";
import { anyGalleryFilter, categoryOptions, filterForSource, filterGallery, groupOptions, type ICategoryOption, type IGalleryFilter, type IGalleryTile, type IGroupOption, NO_GALLERY_FILTER } from "../../gallery";
import type { messages } from "../ArtBrowser.messages";
import { TileGrid } from "../TileGrid";
import type { IGallerySource } from "../useGalleryCatalog";
import { ActiveFilters, CategoryChips, SearchField, StoryPicker } from "./GalleryFilters";

/** One gallery source read through the author's filter: what the rail, the chips and the grid each show. */
export interface IGalleryView {
    source: GalleryKind;
    status: IGallerySource["status"];
    retry: () => void;
    /** The author's filter read against this source (`filterForSource`): a part it lacks is set aside, not cleared. */
    filter: IGalleryFilter;
    tiles: readonly IGalleryTile[];
    categories: readonly ICategoryOption[];
    groups: readonly IGroupOption[];
    /** How many tiles "All stories" leaves. */
    groupTotal: number;
    groupName: string | null;
}

/**
 * The derived state of the gallery browser for one source. `chosen` is one filter across
 * the three sources, so a category or story set on one applies again on the source that
 * has it, and a source never opens empty because of a filter set on another.
 */
export function useGalleryView(source: GalleryKind, catalog: IGallerySource, chosen: IGalleryFilter): IGalleryView {
    const all = catalog.tiles;
    const filter = useMemo(() => filterForSource(all, chosen), [all, chosen]);
    const tiles = useMemo(() => filterGallery(all, filter), [all, filter]);
    const categories = useMemo(() => categoryOptions(all, filter), [all, filter]);
    const groups = useMemo(() => groupOptions(all, filter), [all, filter]);
    const groupTotal = useMemo(() => filterGallery(all, { ...filter, group: null }).length, [all, filter]);
    const groupName = filter.group === null ? null : (all.find((tile) => tile.groupId === filter.group)?.groupName ?? filter.group);
    return { source, status: catalog.status, retry: catalog.retry, filter, tiles, categories, groups, groupTotal, groupName };
}

/** The filter edits the browser offers, over the author's `chosen` filter. */
export function galleryFilterActions(setChosen: (update: (prev: IGalleryFilter) => IGalleryFilter) => void) {
    const update = (part: Partial<IGalleryFilter>) => setChosen((prev) => ({ ...prev, ...part }));
    return {
        update,
        // Toggles the clicked category in the author's own filter, not in the view's `filter`: that one has the categories set aside for another source dropped, and writing it back would lose them (`filterForSource`).
        toggleCategory: (id: StoryCategory) => setChosen((prev) => ({ ...prev, categories: prev.categories.includes(id) ? prev.categories.filter((c) => c !== id) : [...prev.categories, id] })),
        clear: () => setChosen(() => NO_GALLERY_FILTER),
    };
}

interface IGalleryBarProps {
    view: IGalleryView;
    query: string;
    actions: ReturnType<typeof galleryFilterActions>;
}

/**
 * The gallery's filter bar, part of the browser's sticky header: the search, the story
 * picker and the count on one row, the category chips and any active story and Clear all
 * on the next. Nothing but the search shows until the catalogue has loaded.
 */
export function GalleryBar({ view, query, actions }: IGalleryBarProps) {
    const t: TypedT<typeof messages> = useT("user");
    const ready = view.status === "success";
    const { filter } = view;
    const filtering = anyGalleryFilter(filter);
    return (
        <>
            <div className="flex min-w-0 items-center gap-2">
                <SearchField value={query} onChange={(next) => actions.update({ query: next })} placeholder={t("profile.background.gallery.searchPlaceholder")} label={t("profile.background.gallery.searchLabel")} className="min-w-0 flex-1" />
                {ready && <StoryPicker groups={view.groups} chosen={filter.group} chosenName={view.groupName} total={view.groupTotal} onChoose={(group) => actions.update({ group })} />}
            </div>
            {ready && (
                <div className="flex min-w-0 flex-wrap items-center gap-1.5">
                    {view.categories.length > 1 && <CategoryChips options={view.categories} chosen={filter.categories} onToggle={actions.toggleCategory} />}
                    {filtering && <ActiveFilters groupName={view.groupName} onRemoveGroup={() => actions.update({ group: null })} onClear={actions.clear} />}
                    <span className="ms-auto font-sans text-muted-foreground text-xs tabular-nums" aria-live="polite">
                        {t("profile.background.gallery.count", { count: view.tiles.length })}
                    </span>
                </div>
            )}
        </>
    );
}

/**
 * The results' header beside the rail: the search, the count and `end` (the tile-size
 * control) on one row; the chosen story's chip and Clear all under it while filtering.
 */
export function GalleryResultsHeader({ view, query, actions, end }: IGalleryBarProps & { end?: ReactNode }) {
    const t: TypedT<typeof messages> = useT("user");
    const filtering = anyGalleryFilter(view.filter);
    return (
        <>
            <div className="flex min-w-0 items-center gap-2">
                <SearchField value={query} onChange={(next) => actions.update({ query: next })} placeholder={t("profile.background.gallery.searchPlaceholder")} label={t("profile.background.gallery.searchLabel")} className="min-w-0 flex-1" />
                {view.status === "success" && (
                    <span className="shrink-0 px-1 font-sans text-muted-foreground text-xs tabular-nums" aria-live="polite">
                        {t("profile.background.gallery.count", { count: view.tiles.length })}
                    </span>
                )}
                {end}
            </div>
            {view.status === "success" && filtering && (
                <div className="flex min-w-0 flex-wrap items-center gap-1.5">
                    <ActiveFilters groupName={view.groupName} onRemoveGroup={() => actions.update({ group: null })} onClear={actions.clear} />
                </div>
            )}
        </>
    );
}

interface IGalleryTilesProps {
    view: IGalleryView;
    selected: { kind: GalleryKind; id: string } | null;
    onPick: (kind: GalleryKind, id: string) => void;
    /** The smallest tile width, in CSS px. */
    minTile: number;
}

/** The gallery's tiles, or the load's progress or failure in their place. They flow in the editor's scroller. */
export function GalleryTiles({ view, selected, onPick, minTile }: IGalleryTilesProps) {
    const t: TypedT<typeof messages> = useT("user");
    if (view.status === "pending") {
        return (
            <div className="flex items-center justify-center gap-2 py-16 font-sans text-muted-foreground text-sm">
                <Spinner />
                <span>{t("profile.background.gallery.loading")}</span>
            </div>
        );
    }
    if (view.status === "error") {
        return (
            <div className="flex flex-col items-center justify-center gap-3 py-16 font-sans text-muted-foreground text-sm">
                <span>{t("profile.background.gallery.error")}</span>
                <Button type="button" variant="outline" size="sm" onClick={view.retry}>
                    <RotateCwIcon />
                    {t("profile.background.gallery.retry")}
                </Button>
            </div>
        );
    }
    const { filter } = view;
    const pagingKey = `${view.source}\u0000${filter.categories.join(",")}\u0000${filter.group ?? ""}\u0000${filter.query}`;
    return (
        <div className="flex flex-col">
            {view.tiles.length === 0 ? <p className="m-0 py-16 text-center font-sans text-muted-foreground text-sm">{t("profile.background.gallery.empty")}</p> : <TileGrid key={view.source} tiles={view.tiles} selected={selected} onPick={onPick} pagingKey={pagingKey} minTile={minTile} />}
        </div>
    );
}
