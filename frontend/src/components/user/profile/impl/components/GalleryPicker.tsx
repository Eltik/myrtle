import { useQuery } from "@tanstack/react-query";
import { CheckIcon, ChevronDownIcon, FilterXIcon, RotateCwIcon, SearchIcon, XIcon } from "lucide-react";
import { type KeyboardEvent, type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { FacetFilter } from "#/components/tier-lists/edit/FacetFilter";
import type { IPoolFacet } from "#/components/tier-lists/edit/poolKinds";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Spinner } from "#/components/ui/spinner";
import { storyArtGalleryQueryOptions, storyGalleryQueryOptions } from "#/lib/api/story";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import { GALLERY_KINDS, type GalleryKind, galleryThumbUrl } from "../background";
import { anyGalleryFilter, archiveTiles, categoryOptions, filterForSource, filterGallery, groupOptions, type ICategoryOption, type IGalleryFilter, type IGalleryTile, type IGroupOption, NO_GALLERY_FILTER, pagesFor, searchGroups, storyArtTiles, visibleTiles } from "../gallery";
import type { messages } from "./BackgroundPicker.messages";

interface IGalleryPickerProps {
    /** The gallery picture the draft shows, `null` when the draft is no gallery picture. */
    selected: { kind: GalleryKind; id: string } | null;
    onPick: (kind: GalleryKind, id: string) => void;
    /** A rail of sources, categories and stories beside the tiles; else the stacked rows of a narrow picker or the phone sheet. */
    wide: boolean;
}

/** A library category's label, each key written out so the extractor sees it. */
function categoryLabel(t: TypedT<typeof messages>, category: StoryCategory): string {
    switch (category) {
        case "main":
            return t("profile.background.gallery.category.main");
        case "side":
            return t("profile.background.gallery.category.side");
        case "vignette":
            return t("profile.background.gallery.category.vignette");
        case "is":
            return t("profile.background.gallery.category.is");
        case "reclamation":
            return t("profile.background.gallery.category.reclamation");
        case "sideContent":
            return t("profile.background.gallery.category.sideContent");
        case "record":
            return t("profile.background.gallery.category.record");
    }
}

/** A source's label. */
function sourceLabel(t: TypedT<typeof messages>, kind: GalleryKind): string {
    switch (kind) {
        case "archive_pic":
            return t("profile.background.gallery.source.archive_pic");
        case "story_cg":
            return t("profile.background.gallery.source.story_cg");
        case "story_scene":
            return t("profile.background.gallery.source.story_scene");
    }
}

const PILL = "inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-3 font-medium font-sans text-xs leading-none transition-colors";
const pill = (active: boolean) => cn(PILL, active ? "border-primary bg-primary text-primary-foreground" : "border-border bg-popover text-muted-foreground hover:bg-accent hover:text-foreground");
/** A rail row: a full-width button with its count on the right. */
const railRow = (active: boolean) => cn("flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-start font-sans text-[13px] leading-tight transition-colors", active ? "bg-primary/12 font-medium text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground");
/** The count at the end of a rail row. */
const RAIL_COUNT = "font-mono text-[11px] tabular-nums opacity-70";
/** The tier-list editor's facet heading style, shared by the rail's headings. */
const RAIL_HEADING = "m-0 mb-1.5 px-2 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]";

/**
 * The background picker's Gallery tab: three sources (the Archives gallery, the story CGs,
 * the story scene plates), each a grid of 16:9 tiles of the backend's 320 px JPEGs. A source
 * narrows by library category (several at once, the tier-list editor's facet toggles), by
 * one story out of a searchable list, and by a `fuzzy.ts` search; the three combine, each
 * shows the count it would leave, and the active ones show as chips above the tiles. A wide
 * picker puts the sources, categories and stories in a left rail; a narrow one stacks them
 * in rows, the story list behind a button.
 *
 * The grid mounts 120 tiles (`TILE_PAGE`) and adds a page as its end scrolls into view, so
 * the 1,230 CGs never mount at once; on open it mounts as many pages as the saved pick
 * needs and scrolls it into view. Arrow keys move between tiles and Enter picks one. The
 * catalogues are the default server's: the picture routes fall back to it from any other,
 * and the header asks for it the same way.
 */
export function GalleryPicker({ selected, onPick, wide }: IGalleryPickerProps) {
    const t: TypedT<typeof messages> = useT("user");
    const archive = useQuery(storyGalleryQueryOptions());
    const cgs = useQuery(storyArtGalleryQueryOptions("cg"));
    const scenes = useQuery(storyArtGalleryQueryOptions("scene"));
    const [source, setSource] = useState<GalleryKind>(selected?.kind ?? "archive_pic");
    // One filter across the sources: a part the shown source lacks is set aside (`filterForSource`), not cleared, so it applies again on the source that has it.
    const [chosen, setChosen] = useState<IGalleryFilter>(NO_GALLERY_FILTER);

    const queries = { archive_pic: archive, story_cg: cgs, story_scene: scenes } as const;
    const active = queries[source];
    const tilesBySource = useMemo(
        () => ({
            archive_pic: archive.data ? archiveTiles(archive.data) : [],
            story_cg: cgs.data ? storyArtTiles(cgs.data) : [],
            story_scene: scenes.data ? storyArtTiles(scenes.data) : [],
        }),
        [archive.data, cgs.data, scenes.data],
    );
    const all = tilesBySource[source];
    const filter = useMemo(() => filterForSource(all, chosen), [all, chosen]);
    const tiles = useMemo(() => filterGallery(all, filter), [all, filter]);
    const categories = useMemo(() => categoryOptions(all, filter), [all, filter]);
    const groups = useMemo(() => groupOptions(all, filter), [all, filter]);
    const groupName = filter.group === null ? null : (all.find((tile) => tile.groupId === filter.group)?.groupName ?? filter.group);

    // Paging is keyed on what the tiles are, so a new source, filter or search starts at one page again.
    const pagingKey = `${source}\u0000${filter.categories.join(",")}\u0000${filter.group ?? ""}\u0000${filter.query}`;
    const [paging, setPaging] = useState({ key: "", pages: 1 });
    // On open the saved pick is revealed once: enough pages mount to hold it, and it scrolls into view.
    const [revealing, setRevealing] = useState(selected !== null);
    const selectedIndex = selected && selected.kind === source ? tiles.findIndex((tile) => tile.id === selected.id) : -1;
    const pages = paging.key === pagingKey ? paging.pages : revealing ? pagesFor(selectedIndex) : 1;
    const shown = visibleTiles(tiles, pages);

    const gridRef = useRef<HTMLUListElement>(null);
    useEffect(() => {
        if (!revealing || active.status === "pending") return;
        setRevealing(false);
        if (selectedIndex < 0) return;
        setPaging({ key: pagingKey, pages });
        // Only the tile grid's own scroller moves: `scrollIntoView` would also scroll the picker body and the rail, taking the search and filters out of view.
        const tile = gridRef.current?.querySelector<HTMLElement>("[aria-pressed=true]");
        const scroller = tile?.closest<HTMLElement>("[data-slot=scroll-area-viewport]");
        if (tile && scroller) scrollWithin(scroller, tile, "center");
    }, [revealing, active.status, selectedIndex, pagingKey, pages]);

    const sentinel = useRef<HTMLLIElement>(null);
    useEffect(() => {
        const node = sentinel.current;
        if (!node || shown.length >= tiles.length) return;
        const observer = new IntersectionObserver((entries) => entries.some((e) => e.isIntersecting) && setPaging({ key: pagingKey, pages: pages + 1 }), { rootMargin: "400px" });
        observer.observe(node);
        return () => observer.disconnect();
    }, [shown.length, tiles.length, pagingKey, pages]);

    const sourceCount = (kind: GalleryKind) => (queries[kind].data ? tilesBySource[kind].length : null);
    const update = (part: Partial<IGalleryFilter>) => setChosen((prev) => ({ ...prev, ...part }));
    const toggleCategory = (id: StoryCategory) => update({ categories: filter.categories.includes(id) ? filter.categories.filter((c) => c !== id) : [...filter.categories, id] });
    const ready = active.status === "success" && active.data !== null;

    const sources = wide ? (
        <section>
            <h3 className={RAIL_HEADING}>{t("profile.background.gallery.sources")}</h3>
            <div className="flex flex-col gap-0.5">
                {GALLERY_KINDS.map((kind) => (
                    <RailRow key={kind} active={source === kind} onClick={() => setSource(kind)} name={sourceLabel(t, kind)} count={sourceCount(kind) ?? ""} />
                ))}
            </div>
        </section>
    ) : (
        <fieldset className="m-0 -mx-6 min-w-0 overflow-x-auto border-0 p-0 px-6 max-sm:-mx-4 max-sm:px-4" aria-label={t("profile.background.gallery.sources")}>
            <div className="flex w-max gap-1.5">
                {GALLERY_KINDS.map((kind) => {
                    const count = sourceCount(kind);
                    return (
                        <button key={kind} type="button" aria-pressed={source === kind} onClick={() => setSource(kind)} className={cn(pill(source === kind), "font-semibold")}>
                            {sourceLabel(t, kind)}
                            {count !== null && <span className="tabular-nums opacity-70">{count}</span>}
                        </button>
                    );
                })}
            </div>
        </fieldset>
    );

    const storyList = <StoryList t={t} groups={groups} chosen={filter.group} total={filterGallery(all, { ...filter, group: null }).length} onChoose={(group) => update({ group })} />;

    const status =
        active.status === "pending" ? (
            <div className="flex min-h-0 flex-1 items-center justify-center gap-2 font-sans text-muted-foreground text-sm">
                <Spinner />
                <span>{t("profile.background.gallery.loading")}</span>
            </div>
        ) : !ready ? (
            <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 font-sans text-muted-foreground text-sm">
                <span>{t("profile.background.gallery.error")}</span>
                <Button type="button" variant="outline" size="sm" onClick={() => void active.refetch()}>
                    <RotateCwIcon />
                    {t("profile.background.gallery.retry")}
                </Button>
            </div>
        ) : null;

    // `beside` sits on the search's row: the narrow layout's story button, so it costs no row of its own.
    const results = (beside?: ReactNode) => (
        <>
            <div className="flex min-w-0 items-center gap-2">
                <InputGroup className="min-w-0 flex-1">
                    <InputGroupAddon>
                        <SearchIcon aria-hidden="true" />
                    </InputGroupAddon>
                    <InputGroupInput value={chosen.query} onChange={(e) => update({ query: (e.target as HTMLInputElement).value })} placeholder={t("profile.background.gallery.searchPlaceholder")} type="search" aria-label={t("profile.background.gallery.searchLabel")} />
                </InputGroup>
                {beside}
            </div>
            <ActiveFilters t={t} filter={filter} count={tiles.length} groupName={groupName} onRemoveCategory={toggleCategory} onRemoveGroup={() => update({ group: null })} onClear={() => setChosen(NO_GALLERY_FILTER)} />
            <div className="relative min-h-0 flex-1">
                {tiles.length === 0 ? (
                    <div className="flex h-full items-center justify-center font-sans text-muted-foreground text-sm">{t("profile.background.gallery.empty")}</div>
                ) : (
                    <ScrollArea className="h-full">
                        <ul ref={gridRef} aria-label={t("profile.background.gallery.grid")} onKeyDown={moveFocus} className={cn("m-0 grid list-none gap-2 p-0 pe-2", wide ? "grid-cols-[repeat(auto-fill,minmax(176px,1fr))]" : "grid-cols-[repeat(auto-fill,minmax(150px,1fr))]")}>
                            {shown.map((tile) => (
                                <GalleryTile key={tile.id} t={t} tile={tile} isSelected={selected !== null && tile.kind === selected.kind && tile.id === selected.id} onPick={onPick} />
                            ))}
                            {shown.length < tiles.length && <li ref={sentinel} aria-hidden="true" className="col-span-full h-px" />}
                        </ul>
                    </ScrollArea>
                )}
            </div>
        </>
    );

    if (wide) {
        return (
            <div className="flex min-h-0 flex-1 gap-4 px-6 pb-3">
                {/* Size containment, as the picker body gives the tile column: the story list's own height must not count toward the body's, or the body grows to it and scrolls instead of the rail. */}
                <aside className="flex w-56 shrink-0 flex-col gap-4 overflow-y-auto border-e pe-3 contain-size">
                    {sources}
                    {ready && categories.length > 1 && <CategoryRail t={t} options={categories} chosen={filter.categories} onToggle={toggleCategory} />}
                    {ready && (
                        <section className="flex min-h-0 flex-1 flex-col">
                            <h3 className={RAIL_HEADING}>{t("profile.background.gallery.stories")}</h3>
                            {storyList}
                        </section>
                    )}
                </aside>
                <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-3">{status ?? results()}</div>
            </div>
        );
    }

    return (
        <div className="flex min-h-0 flex-1 flex-col gap-3 px-6 pb-3 max-sm:px-4">
            {sources}
            {status ?? (
                <>
                    {categories.length > 1 && <CategoryFacet t={t} options={categories} chosen={filter.categories} onChange={(next) => update({ categories: next })} />}
                    {results(
                        <Popover>
                            <PopoverTrigger render={<Button type="button" variant="outline" className="max-w-[45%] shrink-0" />}>
                                <span className="min-w-0 truncate">{t("profile.background.gallery.storyButton", { name: groupName ?? t("profile.background.gallery.allGroups") })}</span>
                                <ChevronDownIcon aria-hidden="true" />
                            </PopoverTrigger>
                            <PopoverPopup align="end" className="h-80 w-72 max-w-[calc(100vw-2rem)] [&_[data-slot=popover-viewport]]:flex [&_[data-slot=popover-viewport]]:flex-col [&_[data-slot=popover-viewport]]:py-3">
                                {storyList}
                            </PopoverPopup>
                        </Popover>,
                    )}
                </>
            )}
        </div>
    );
}

/** The tile-grid keys that move within a row or to either end; Up and Down step by the column count. */
const ROW_STEPS: Record<string, number | "start" | "end"> = { ArrowLeft: -1, ArrowRight: 1, Home: "start", End: "end" };

/**
 * Arrow keys between the tiles of a grid of buttons, by its rendered column count; Home and
 * End jump to the first and last mounted tile. Enter and Space are the buttons' own.
 */
function moveFocus(e: KeyboardEvent<HTMLUListElement>) {
    const grid = e.currentTarget;
    const columns = getComputedStyle(grid).gridTemplateColumns.split(" ").filter(Boolean).length || 1;
    const step = e.key === "ArrowUp" ? -columns : e.key === "ArrowDown" ? columns : ROW_STEPS[e.key];
    if (step === undefined) return;
    const buttons = [...grid.querySelectorAll<HTMLButtonElement>("button[data-tile]")];
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (at < 0) return;
    const next = step === "start" ? 0 : step === "end" ? buttons.length - 1 : at + step;
    const target = buttons[next];
    if (!target) return;
    e.preventDefault();
    target.focus({ preventScroll: true });
    const scroller = target.closest<HTMLElement>("[data-slot=scroll-area-viewport]");
    if (scroller) scrollWithin(scroller, target, "nearest");
}

/** Scrolls `scroller` alone so `el` shows: centred, or by the least distance that brings it fully in. */
function scrollWithin(scroller: HTMLElement, el: HTMLElement, block: "center" | "nearest") {
    const box = scroller.getBoundingClientRect();
    const r = el.getBoundingClientRect();
    if (block === "center") {
        scroller.scrollTop += r.top - box.top - (box.height - r.height) / 2;
        return;
    }
    if (r.top < box.top) scroller.scrollTop += r.top - box.top;
    else if (r.bottom > box.bottom) scroller.scrollTop += r.bottom - box.bottom;
}

function GalleryTile({ t, tile, isSelected, onPick }: { t: TypedT<typeof messages>; tile: IGalleryTile; isSelected: boolean; onPick: (kind: GalleryKind, id: string) => void }) {
    const title = tile.title || tile.groupName;
    const label = tile.title ? t("profile.background.gallery.tileLabel", { title, group: tile.groupName }) : t("profile.background.gallery.storyTileLabel", { group: tile.groupName });
    return (
        <li>
            <button
                type="button"
                data-tile=""
                onClick={() => onPick(tile.kind, tile.id)}
                aria-label={label}
                aria-pressed={isSelected}
                title={label}
                className={cn("group flex w-full cursor-pointer flex-col overflow-hidden rounded-md border bg-card text-start outline-none transition-colors hover:border-primary focus-visible:ring-2 focus-visible:ring-ring", isSelected ? "border-primary ring-2 ring-primary/50" : "border-border")}
            >
                <span className="relative block aspect-video w-full overflow-hidden bg-[oklch(0.2_0.005_285)]">
                    <img src={galleryThumbUrl(tile.kind, tile.id)} alt="" loading="lazy" decoding="async" draggable={false} width={320} height={180} className="absolute inset-0 h-full w-full object-cover" />
                    {isSelected && (
                        <span className="absolute top-1.5 right-1.5 inline-flex size-5 items-center justify-center rounded-full bg-primary text-primary-foreground shadow" aria-hidden="true">
                            <CheckIcon className="size-3.5" strokeWidth={3} />
                        </span>
                    )}
                </span>
                <span className={cn("line-clamp-1 px-1.5 py-1 font-medium font-sans text-[11px] leading-tight", isSelected ? "text-primary" : "text-foreground")}>{title}</span>
            </button>
        </li>
    );
}

/** The narrow picker's category row: the tier-list editor's `FacetFilter`, each toggle carrying its count. */
function CategoryFacet({ t, options, chosen, onChange }: { t: TypedT<typeof messages>; options: readonly ICategoryOption[]; chosen: readonly StoryCategory[]; onChange: (next: StoryCategory[]) => void }) {
    const facet: IPoolFacet = {
        id: "category",
        label: t("profile.background.gallery.categories"),
        groupLabel: t("profile.background.gallery.categoriesLabel"),
        variant: "text",
        options: options.map((option) => ({ value: option.id, label: `${categoryLabel(t, option.id)} ${option.count}`, ariaLabel: t("profile.background.gallery.categoryOption", { label: categoryLabel(t, option.id), count: option.count }) })),
        // The row only renders the options; the tiles are matched by `filterGallery`.
        valueOf: () => null,
    };
    const known = new Set<string>(options.map((option) => option.id));
    return <FacetFilter facet={facet} value={[...chosen]} onChange={(next) => onChange(next.filter((v): v is StoryCategory => known.has(v)))} />;
}

/** The wide picker's category toggles, one rail row each with its count; several can be on. */
function CategoryRail({ t, options, chosen, onToggle }: { t: TypedT<typeof messages>; options: readonly ICategoryOption[]; chosen: readonly StoryCategory[]; onToggle: (id: StoryCategory) => void }) {
    return (
        <section>
            <h3 className={RAIL_HEADING}>{t("profile.background.gallery.categories")}</h3>
            <fieldset className="m-0 flex min-w-0 flex-col gap-0.5 border-0 p-0" aria-label={t("profile.background.gallery.categoriesLabel")}>
                {options.map((option) => {
                    const on = chosen.includes(option.id);
                    return (
                        <button key={option.id} type="button" aria-pressed={on} onClick={() => onToggle(option.id)} aria-label={t("profile.background.gallery.categoryOption", { label: categoryLabel(t, option.id), count: option.count })} className={railRow(on)}>
                            <span className={cn("inline-flex size-4 shrink-0 items-center justify-center rounded-[4px] border", on ? "border-primary bg-primary text-primary-foreground" : "border-input")} aria-hidden="true">
                                {on && <CheckIcon className="size-3" strokeWidth={3} />}
                            </span>
                            <span className={cn("min-w-0 flex-1 truncate", option.count === 0 && !on && "opacity-55")}>{categoryLabel(t, option.id)}</span>
                            <span className={RAIL_COUNT}>{option.count}</span>
                        </button>
                    );
                })}
            </fieldset>
        </section>
    );
}

/** A rail row that picks one thing: its name, and its count at the end. */
function RailRow({ active, onClick, name, count, title }: { active: boolean; onClick: () => void; name: string; count: number | string; title?: string }) {
    return (
        <button type="button" aria-pressed={active} onClick={onClick} title={title} className={railRow(active)}>
            <span className="min-w-0 flex-1 truncate">{name}</span>
            <span className={RAIL_COUNT}>{count}</span>
        </button>
    );
}

/** The story list: its own search (through `fuzzy.ts`), "All stories" first, then each story with its count. */
function StoryList({ t, groups, chosen, total, onChoose }: { t: TypedT<typeof messages>; groups: readonly IGroupOption[]; chosen: string | null; total: number; onChoose: (group: string | null) => void }) {
    const [query, setQuery] = useState("");
    const listed = useMemo(() => searchGroups(groups, query), [groups, query]);
    return (
        <div className="flex min-h-0 flex-1 flex-col gap-2">
            <InputGroup className="shrink-0">
                <InputGroupAddon>
                    <SearchIcon aria-hidden="true" />
                </InputGroupAddon>
                <InputGroupInput value={query} onChange={(e) => setQuery((e.target as HTMLInputElement).value)} placeholder={t("profile.background.gallery.storySearchPlaceholder")} type="search" aria-label={t("profile.background.gallery.storySearchLabel")} />
            </InputGroup>
            <div className="-me-2 flex min-h-24 flex-1 flex-col gap-0.5 overflow-y-auto pe-2">
                {query.trim().length === 0 && <RailRow active={chosen === null} onClick={() => onChoose(null)} name={t("profile.background.gallery.allGroups")} count={total} />}
                {listed.map((group) => (
                    <RailRow key={group.id} active={chosen === group.id} onClick={() => onChoose(chosen === group.id ? null : group.id)} title={group.name} name={group.name} count={group.count} />
                ))}
                {listed.length === 0 && <p className="m-0 px-2 py-1.5 font-sans text-muted-foreground text-xs">{t("profile.background.gallery.storyEmpty")}</p>}
            </div>
        </div>
    );
}

/** The result count, a removable chip per active category and story, and Clear all when anything filters. */
function ActiveFilters({ t, filter, count, groupName, onRemoveCategory, onRemoveGroup, onClear }: { t: TypedT<typeof messages>; filter: IGalleryFilter; count: number; groupName: string | null; onRemoveCategory: (id: StoryCategory) => void; onRemoveGroup: () => void; onClear: () => void }) {
    const chips: { key: string; name: string; onRemove: () => void }[] = [
        ...filter.categories.map((id) => ({ key: `c:${id}`, name: categoryLabel(t, id), onRemove: () => onRemoveCategory(id) })),
        ...(filter.group !== null && groupName !== null ? [{ key: `g:${filter.group}`, name: groupName, onRemove: onRemoveGroup }] : []),
    ];
    return (
        <div className="-mt-1 flex min-h-6 flex-wrap items-center gap-1.5 font-sans text-xs" aria-live="polite">
            <span className="me-1 text-muted-foreground tabular-nums">{t("profile.background.gallery.count", { count })}</span>
            {chips.map((chip) => (
                <span key={chip.key} className="inline-flex h-6 max-w-56 items-center gap-1 rounded-full border border-primary/40 bg-primary/10 ps-2.5 pe-0.5 text-foreground">
                    <span className="min-w-0 truncate">{chip.name}</span>
                    <button type="button" onClick={chip.onRemove} aria-label={t("profile.background.gallery.removeFilter", { name: chip.name })} className="inline-flex size-5 cursor-pointer items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-foreground">
                        <XIcon className="size-3" />
                    </button>
                </span>
            ))}
            {anyGalleryFilter(filter) && (
                <Button type="button" variant="ghost" size="xs" className="ms-auto" onClick={onClear}>
                    <FilterXIcon />
                    {t("profile.background.gallery.clearFilters")}
                </Button>
            )}
        </div>
    );
}
