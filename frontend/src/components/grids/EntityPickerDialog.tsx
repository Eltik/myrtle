import { useQuery } from "@tanstack/react-query";
import { FilterXIcon, RotateCwIcon, SearchIcon, Trash2Icon } from "lucide-react";
import { type RefObject, useEffect, useMemo, useRef, useState } from "react";
import { FacetFilter, useEdgeFade } from "#/components/tier-lists/edit/FacetFilter";
import { anyFacetSelected, type FacetSelection, matchesFacets, matchesSearch } from "#/components/tier-lists/edit/poolFilters";
import { usePoolKind } from "#/components/tier-lists/edit/poolKinds";
import { EntityAvatar } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Spinner } from "#/components/ui/spinner";
import { TooltipProvider } from "#/components/ui/tooltip";
import { type ITierEntity, type TierEntityKind, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { tierEntityCatalogueQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import type { messages } from "./EntityPickerDialog.messages";
import { orderKinds } from "./shared";
import { type IPickerTarget, initialPickerKind } from "./state";

/** Tiles rendered at first and per "Show more": the enemy catalogue alone is ~1,540 entries. */
const PAGE_SIZE = 180;
/**
 * The tab the author used last, remembered for this page load only. It used
 * to live in localStorage, so a tab from an earlier session decided the first
 * open of a new one; a fresh page now opens on the grid's first allowed type.
 */
let lastKind: TierEntityKind | null = null;
/** Where the Operators tab finds operators the reader's server has not released yet. */
const PREVIEW_SERVER = "cn";
/** Servers that already serve CN's roster: no preview operators to add. */
const CN_ROSTER_SERVERS: readonly string[] = [PREVIEW_SERVER, "bili"];

interface IEntityPickerDialogProps {
    /** The cell being picked for, `null` when closed. */
    target: IPickerTarget | null;
    /** The grid's allowed types: the only tabs offered, in `ALL_ENTITY_KINDS` order. */
    kinds: readonly TierEntityKind[];
    onClose: () => void;
    /** `server` is where the pick came from when it is not the reader's server (a CN-only operator), else `null`. */
    onPick: (entity: ITierEntity, server: string | null) => void;
    onClear: () => void;
}

export function EntityPickerDialog({ target, kinds, onClose, onPick, onClear }: IEntityPickerDialogProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const open = target !== null;
    const searchRef = useRef<HTMLInputElement | null>(null);

    return (
        <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
            {/* A phone gets every pixel under the dialog viewport's 3rem top inset: the tiles are what the space is for. */}
            <DialogPopup className="h-[min(85dvh,760px)] max-sm:h-[calc(100dvh-3rem)] sm:max-w-3xl" initialFocus={searchRef}>
                <DialogHeader>
                    <DialogTitle>{target?.label ? t("picker.titleLabelled", { label: target.label }) : t("picker.title", { row: target?.row ?? 1, col: target?.col ?? 1 })}</DialogTitle>
                    <DialogDescription className="max-sm:sr-only">{t("picker.description")}</DialogDescription>
                </DialogHeader>
                {target && <EntityPickerBody kinds={kinds} current={target.current} onPick={onPick} searchRef={searchRef} />}
                <DialogFooter className="justify-between max-sm:flex-row max-sm:py-3 sm:justify-between">
                    <Button type="button" variant="ghost" onClick={onClear} disabled={!target?.hasPick}>
                        <Trash2Icon />
                        {t("picker.clear")}
                    </Button>
                    <DialogClose render={<Button type="button" variant="outline" />}>{t("picker.cancel")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

interface IEntityPickerBodyProps {
    kinds: readonly TierEntityKind[];
    current: ITierEntity | null;
    /** Every entity already chosen, by `entity.key`, for a picker that collects several (a profile's favourites). Marks them as `current` marks one. */
    selected?: ReadonlySet<string>;
    onPick: (entity: ITierEntity, server: string | null) => void;
    /** The search input, for the caller's dialog to focus on open (its `initialFocus`). */
    searchRef?: RefObject<HTMLInputElement | null>;
}

/** No facet selected: what every kind starts with. */
const NO_FACETS: FacetSelection = {};

/** A catalogue entry as an offered, unplaced entity. */
function catalogueEntity(summary: EntitySummary): ITierEntity {
    return toTierEntity(summary.kind, summary.id, summary, UNPLACED);
}

/** The picker's tabs, search and tiles, for a dialog of the caller's own. */
export function EntityPickerBody({ kinds, current, selected, onPick, searchRef }: IEntityPickerBodyProps) {
    const t: TypedT<typeof messages> = useT("grids");
    const labels = useEntityLabels();
    const server = useGamedataServer();
    const tabs = useMemo(() => orderKinds(kinds), [kinds]);
    const [kind, setKind] = useState<TierEntityKind>(() => initialPickerKind(tabs, current?.kind ?? null, lastKind));
    const [query, setQuery] = useState("");
    const [limit, setLimit] = useState(PAGE_SIZE);
    // Each tab keeps its own facet selections while the picker is open, so a tab visited again filters as it was left; a fresh open starts unfiltered.
    const [facetsByKind, setFacetsByKind] = useState<Partial<Record<TierEntityKind, FacetSelection>>>({});
    const facets = facetsByKind[kind] ?? NO_FACETS;
    const ownSearchRef = useRef<HTMLInputElement | null>(null);
    const search = searchRef ?? ownSearchRef;
    const tabsFade = useEdgeFade<HTMLDivElement>();

    const catalogue = useQuery(tierEntityCatalogueQueryOptions(kind, server));
    // The Operators tab also offers what CN has released and this server has not, after the rest. A failed load leaves the tab as it was.
    const withPreview = kind === "operator" && !CN_ROSTER_SERVERS.includes(server);
    const previewCatalogue = useQuery({ ...tierEntityCatalogueQueryOptions("operator", PREVIEW_SERVER), enabled: withPreview });
    const released = useMemo(() => catalogue.data?.map(catalogueEntity) ?? [], [catalogue.data]);
    const preview = useMemo(() => {
        if (!withPreview || !catalogue.data || !previewCatalogue.data) return [];
        const known = new Set(released.map((entity) => entity.key));
        return previewCatalogue.data.map(catalogueEntity).filter((entity) => !known.has(entity.key));
    }, [withPreview, catalogue.data, previewCatalogue.data, released]);
    // A CN-only operator's name is Chinese; outside CN its romanized appellation names it, as the saved grid's resolver does, and the Chinese name stays searchable.
    const previewNames = useMemo(() => new Map(preview.map((entity) => [entity.key, entity.name])), [preview]);
    const previewTiles = useMemo(() => preview.map((entity) => (entity.resolved && entity.kind === "operator" && entity.appellation?.trim() ? { ...entity, name: entity.appellation } : entity)), [preview]);
    const entities = useMemo(() => (previewTiles.length > 0 ? [...released, ...previewTiles] : released), [released, previewTiles]);
    const pool = usePoolKind(kind, entities);

    const filtered = useMemo(() => {
        const q = compactForSearch(query);
        const ordered = (list: ITierEntity[]) => {
            const matched = list.filter((entity) => matchesFacets(entity, pool.facets, facets) && matchesSearch([...pool.searchTexts(entity), previewNames.get(entity.key) ?? null], q));
            return pool.compare ? matched.sort(pool.compare) : matched;
        };
        return [...ordered(released), ...ordered(previewTiles)];
    }, [released, previewTiles, previewNames, pool, query, facets]);

    // A new tab, search or filter starts the pages over.
    // biome-ignore lint/correctness/useExhaustiveDependencies: `kind`, `query` and `facets` are the triggers, not values read inside
    useEffect(() => setLimit(PAGE_SIZE), [kind, query, facets]);

    const selectKind = (next: TierEntityKind) => {
        setKind(next);
        lastKind = next;
    };
    const setFacet = (id: string, values: string[]) => setFacetsByKind((prev) => ({ ...prev, [kind]: { ...(prev[kind] ?? NO_FACETS), [id]: values } }));
    const clearFacets = () => setFacetsByKind((prev) => ({ ...prev, [kind]: NO_FACETS }));

    const visible = filtered.slice(0, limit);

    return (
        <div className="flex min-h-0 flex-1 flex-col gap-3 px-6 pb-3">
            {/* The vertical padding, cancelled by the margin, is room for a tab's 44 px touch area, which the scroller would clip. */}
            <div ref={tabsFade.ref} style={tabsFade.style} className="-mx-6 -my-1.5 overflow-x-auto px-6 py-1.5 max-sm:[scrollbar-width:none] max-sm:[&::-webkit-scrollbar]:hidden" role="tablist" aria-label={t("picker.kinds")}>
                <div className="flex w-max gap-1.5">
                    {tabs.map((k) => (
                        <button
                            key={k}
                            type="button"
                            role="tab"
                            aria-selected={k === kind}
                            onClick={(e) => {
                                selectKind(k);
                                // A tab clicked with a mouse hands focus back to the search, so the author can type at once; a keyboard user stays on the tabs, and a touch does not raise the keyboard.
                                if (e.detail > 0 && (e.nativeEvent as PointerEvent).pointerType === "mouse") search.current?.focus();
                            }}
                            className={cn(
                                "relative inline-flex h-8 shrink-0 cursor-pointer items-center rounded-full border px-3 font-medium font-sans text-xs leading-none transition-colors pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11",
                                k === kind ? "border-primary bg-primary text-primary-foreground" : "border-border bg-popover text-muted-foreground hover:bg-accent hover:text-foreground",
                            )}
                        >
                            {labels.plural(k)}
                        </button>
                    ))}
                </div>
            </div>

            <InputGroup>
                <InputGroupAddon>
                    <SearchIcon aria-hidden="true" />
                </InputGroupAddon>
                <InputGroupInput ref={search} value={query} onChange={(e) => setQuery((e.target as HTMLInputElement).value)} placeholder={t("picker.searchPlaceholder", { kind: labels.plural(kind) })} type="search" aria-label={pool.searchLabel} />
            </InputGroup>
            {pool.facets.length > 0 && (
                <TooltipProvider delay={300}>
                    <div className="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-x-4 sm:gap-y-2">
                        {pool.facets.map((facet) => (
                            <FacetFilter key={facet.id} facet={facet} value={facets[facet.id] ?? []} onChange={(v) => setFacet(facet.id, v)} />
                        ))}
                        {anyFacetSelected(facets) && (
                            <Button type="button" variant="ghost" size="xs" className="sm:ml-auto" onClick={clearFacets}>
                                <FilterXIcon />
                                {t("picker.clearFilters")}
                            </Button>
                        )}
                    </div>
                </TooltipProvider>
            )}
            {previewTiles.length > 0 && <p className="m-0 -mt-1 font-sans text-muted-foreground text-xs">{t("picker.previewHint", { badge: t("picker.previewBadge") })}</p>}

            <div className="relative min-h-0 flex-1">
                {catalogue.status === "pending" ? (
                    <div className="flex h-full items-center justify-center gap-2 font-sans text-muted-foreground text-sm">
                        <Spinner />
                        <span>{t("picker.loading")}</span>
                    </div>
                ) : catalogue.status === "error" ? (
                    <div className="flex h-full flex-col items-center justify-center gap-3 font-sans text-muted-foreground text-sm">
                        <span>{t("picker.error")}</span>
                        <Button type="button" variant="outline" size="sm" onClick={() => void catalogue.refetch()}>
                            <RotateCwIcon />
                            {t("picker.retry")}
                        </Button>
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="flex h-full items-center justify-center font-sans text-muted-foreground text-sm">{t("picker.empty")}</div>
                ) : (
                    <ScrollArea className="h-full">
                        <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(76px,1fr))] gap-2 p-0 pe-2 max-sm:grid-cols-4">
                            {visible.map((entity) => {
                                const isSelected = selected?.has(entity.key) ?? current?.key === entity.key;
                                const fromPreview = previewNames.has(entity.key);
                                const label = fromPreview ? t("picker.previewLabel", { name: labels.tileLabel(entity) }) : labels.tileLabel(entity);
                                return (
                                    <li key={entity.key}>
                                        <button
                                            type="button"
                                            onClick={() => onPick(entity, fromPreview ? PREVIEW_SERVER : null)}
                                            aria-label={label}
                                            aria-pressed={isSelected}
                                            title={label}
                                            className={cn("group flex w-full cursor-pointer flex-col overflow-hidden rounded-md border bg-card text-start transition-colors hover:border-primary", isSelected ? "border-primary ring-2 ring-primary/40" : "border-border")}
                                        >
                                            <span className="relative flex aspect-square w-full items-center justify-center overflow-hidden bg-[oklch(0.2_0.005_285)] text-lg text-white">
                                                <EntityAvatar entity={entity} face="tile" tone="dark" server={fromPreview ? PREVIEW_SERVER : undefined} />
                                                {fromPreview && (
                                                    <span className="absolute top-1 left-1 rounded-sm bg-black/70 px-1 py-px font-sans font-semibold text-[9px] text-white uppercase leading-none tracking-wide" aria-hidden="true">
                                                        {t("picker.previewBadge")}
                                                    </span>
                                                )}
                                            </span>
                                            {/* Margin, not padding: line-clamp clips at the padding edge, so padding let a third line's top show. The min height nets the margin out. */}
                                            <span className="my-1 line-clamp-2 min-h-[calc(2.4em-8px)] px-1 text-center font-medium font-sans text-[10.5px] text-foreground leading-tight max-sm:text-[11px]">{entity.name}</span>
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                        {filtered.length > limit && (
                            <div className="flex justify-center py-3">
                                <Button type="button" variant="outline" size="sm" onClick={() => setLimit((n) => n + PAGE_SIZE)}>
                                    {t("picker.showMore", { shown: limit, total: filtered.length })}
                                </Button>
                            </div>
                        )}
                    </ScrollArea>
                )}
            </div>
        </div>
    );
}
