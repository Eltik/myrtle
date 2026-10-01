import { CircleSlash2Icon, FilterXIcon, Maximize2Icon, RotateCwIcon, SearchIcon, TriangleAlertIcon } from "lucide-react";
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Field, FieldLabel } from "#/components/ui/field";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Kicker } from "#/components/ui/kicker";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Spinner } from "#/components/ui/spinner";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "#/components/ui/tooltip";
import { entityOwner, type ITierEntity, type TierEntityKind } from "#/lib/api/tier-entities";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import { hasEntityDrag, readEntityDrag } from "./dnd";
import { useAnyDragLifted, usePoolIsOver } from "./drag-controller";
import { EditableOpTile } from "./EditableOpTile";
import styles from "./Editor.module.css";
import type { messages } from "./KindPool.messages";
import { type IPoolFacet, type IPoolKind, usePoolKind } from "./poolKinds";

/** A grid longer than this renders in pages of {@link PAGE} as it scrolls. The operator pool (~410) never pages; the enemy pool (~1,540) does. */
const PAGE_FROM = 600;
/** Tiles per page of a paged grid. */
export const PAGE = 240;
/** How far below the viewport the next page starts loading. */
const LOAD_AHEAD_MARGIN = "320px 0px";

export type CatalogueStatus = "pending" | "error" | "success";

interface IKindPoolProps {
    kind: TierEntityKind;
    /** The kind's catalogue: every entity an editor may place, in the backend's pool order. `undefined` while it loads. */
    entities: ITierEntity[] | undefined;
    status: CatalogueStatus;
    onRetry?: () => void;
    /** Placed entity keys, of every kind. */
    placedKeys: Set<string>;
    onUnplace: (entityKey: string) => void;
    onPickerActivate: (entity: ITierEntity) => void;
    /** The kind tabs, when the list offers more than one kind. */
    tabs?: ReactNode;
    /** Extra header buttons, before Expand. */
    headerActions?: ReactNode;
    /** Override the root container classes. Use when mounting inside a flex/grid layout that needs `flex-1 min-h-0` instead of `h-full`. */
    rootClassName?: string;
}

const EMPTY: ITierEntity[] = [];

/**
 * One kind's pool: search, the kind's facet filters, and the tile grid that is
 * also the drop target for unplacing. Every kind shares this frame; what
 * differs (labels, filters, search fields, order) comes from {@link usePoolKind}.
 */
export function KindPool({ kind, entities, status, onRetry, placedKeys, onUnplace, onPickerActivate, tabs, headerActions, rootClassName }: IKindPoolProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const rt: TypedRichT<typeof messages> = useRichT("tierLists");
    const all = entities ?? EMPTY;
    const config = usePoolKind(kind, all);
    const { query, setQuery, selected, setFacet, hideUsed, setHideUsed, filtered, hasFilters, clearFilters } = usePoolFilters(all, config, placedKeys);
    const { paged, visible, showMore } = usePages(filtered, `${query}\u0000${JSON.stringify(selected)}\u0000${hideUsed}`);
    const [expandedOpen, setExpandedOpen] = useState(false);
    const [mouseDragOver, setMouseDragOver] = useState(false);
    const dropRef = useRef<HTMLElement | null>(null);
    const touchIsOver = usePoolIsOver();
    const anyDragLifted = useAnyDragLifted();
    const isDragOver = touchIsOver || mouseDragOver;
    const totalAvailable = all.length;

    const handleOver = useCallback((e: React.DragEvent) => {
        if (!hasEntityDrag(e)) return;
        e.preventDefault();
        e.dataTransfer.dropEffect = "move";
        setMouseDragOver(true);
    }, []);

    const handleLeave = useCallback((e: React.DragEvent) => {
        const related = e.relatedTarget as Node | null;
        if (related && dropRef.current?.contains(related)) return;
        setMouseDragOver(false);
    }, []);

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            setMouseDragOver(false);
            const payload = readEntityDrag(e);
            if (!payload) return;
            e.preventDefault();
            onUnplace(payload.entityKey);
        },
        [onUnplace],
    );

    const pageFooter = paged && visible.length < filtered.length && (
        <LoadMore onVisible={showMore} shown={visible.length}>
            <p className="m-0 py-3 text-center font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("edit.pool.more", { shown: visible.length, total: filtered.length })}</p>
        </LoadMore>
    );

    return (
        <TooltipProvider delay={300}>
            <div className={cn("flex h-full min-h-0 flex-col gap-3 rounded-xl border border-border bg-card p-3 shadow-[0_1px_2px_oklch(0_0_0/0.04)]", rootClassName)}>
                <header className="flex items-baseline justify-between gap-2">
                    <div className="min-w-0">
                        <Kicker className="mb-0.5">{config.kicker}</Kicker>
                        <p className="m-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">
                            <span className="font-bold text-foreground">{filtered.length}</span>
                            <span className="opacity-70"> / {totalAvailable}</span>
                        </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-0.5">
                        {hasFilters && (
                            <Button type="button" variant="ghost" size="xs" onClick={clearFilters} aria-label={t("edit.pool.clearFilters.label")}>
                                <FilterXIcon />
                                {t("edit.pool.clear")}
                            </Button>
                        )}
                        {headerActions}
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button type="button" variant="outline" size="xs" onClick={() => setExpandedOpen(true)} aria-haspopup="dialog" aria-expanded={expandedOpen}>
                                        <Maximize2Icon />
                                        {t("edit.pool.expand")}
                                    </Button>
                                }
                            />
                            <TooltipContent>{t("edit.pool.expandHint")}</TooltipContent>
                        </Tooltip>
                    </div>
                </header>

                {tabs}

                <PoolSearch label={config.searchLabel} value={query} onChange={setQuery} />

                <section
                    ref={dropRef}
                    data-tl-drop-pool=""
                    className="relative isolate flex min-h-32 flex-1 flex-col overflow-hidden rounded-lg border border-border/70 border-dashed bg-muted/20 transition-colors"
                    data-drag-over={isDragOver || undefined}
                    style={isDragOver ? { borderStyle: "solid", borderColor: "var(--ring)", background: "color-mix(in srgb, var(--ring) 8%, transparent)" } : undefined}
                    onDragOver={handleOver}
                    onDragLeave={handleLeave}
                    onDrop={handleDrop}
                    aria-label={config.dropArea}
                >
                    {isDragOver && (
                        <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
                            <Badge variant="destructive" size="lg" className="gap-1.5 shadow-md">
                                <CircleSlash2Icon className="size-3.5" />
                                {t("edit.pool.dropToUnplace")}
                            </Badge>
                        </div>
                    )}

                    {filtered.length === 0 ? (
                        <PoolEmptyState status={status} onRetry={onRetry} className="py-8" />
                    ) : (
                        <ScrollArea scrollFade scrollbarGutter viewportClassName="p-2">
                            <ul className={styles.poolGrid} aria-label={config.gridLabel}>
                                {visible.map((entry) => {
                                    const placed = placedKeys.has(entry.key);
                                    return (
                                        <li key={entry.key} className="contents">
                                            <EditableOpTile entity={entry} placed={placed} onActivate={(op) => onPickerActivate(op)} />
                                        </li>
                                    );
                                })}
                            </ul>
                            {pageFooter}
                        </ScrollArea>
                    )}
                </section>

                <p className="m-0 font-sans text-muted-foreground text-xs leading-snug">{anyDragLifted ? <span className="font-medium text-foreground">{t("edit.pool.hintDragging")}</span> : t("edit.pool.hintIdle")}</p>
            </div>

            <Dialog open={expandedOpen} onOpenChange={setExpandedOpen}>
                <DialogPopup className="flex max-h-[min(820px,calc(100dvh-3rem))] w-[min(960px,calc(100vw-2rem))] flex-col gap-0 p-0 sm:max-w-none">
                    <DialogHeader className="px-6 pt-5 pb-3">
                        <DialogTitle>{config.dialogTitle}</DialogTitle>
                        <DialogDescription>
                            {rt("edit.pool.dialogSub", {
                                matched: <span className="font-mono text-foreground tabular-nums">{filtered.length}</span>,
                                total: <span className="font-mono tabular-nums">{totalAvailable}</span>,
                            })}
                        </DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-3 border-border border-y bg-muted/30 px-6 py-3">
                        <PoolSearch label={config.searchLabel} value={query} onChange={setQuery} autoFocus />

                        <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:gap-x-4 sm:gap-y-2">
                            {config.facets.map((facet) => (
                                <FacetFilter key={facet.id} facet={facet} value={selected[facet.id] ?? []} onChange={(v) => setFacet(facet.id, v)} />
                            ))}

                            <label className="inline-flex cursor-pointer items-center justify-between gap-2 rounded-md px-1.5 py-1 text-foreground transition-colors hover:bg-accent/40 sm:ml-auto sm:justify-start" htmlFor="pool-hide-used-dialog">
                                <span className="flex flex-col">
                                    <span className="whitespace-nowrap font-medium font-sans text-[12.5px] leading-none">{t("edit.pool.hideUsed")}</span>
                                    <span className="mt-1 whitespace-nowrap text-[11px] text-muted-foreground leading-none">{placedKeys.size > 0 ? t("edit.pool.placedSoFar", { count: placedKeys.size }) : t("edit.pool.nothingPlaced")}</span>
                                </span>
                                <Switch id="pool-hide-used-dialog" checked={hideUsed} onCheckedChange={setHideUsed} />
                            </label>
                        </div>
                    </div>

                    <div className="min-h-0 flex-1 overflow-hidden px-6 py-3">
                        {filtered.length === 0 ? (
                            <PoolEmptyState status={status} onRetry={onRetry} className="py-16" />
                        ) : (
                            <ScrollArea scrollFade={!anyDragLifted} scrollbarGutter viewportClassName="pr-2">
                                <ul className={styles.poolGrid} aria-label={config.gridLabel}>
                                    {visible.map((entry) => {
                                        const placed = placedKeys.has(entry.key);
                                        const owner = entityOwner(entry);
                                        return (
                                            <li key={entry.key} className="contents">
                                                <Tooltip>
                                                    <TooltipTrigger
                                                        render={
                                                            <EditableOpTile
                                                                entity={entry}
                                                                placed={placed}
                                                                onActivate={(op) => {
                                                                    setExpandedOpen(false);
                                                                    onPickerActivate(op);
                                                                }}
                                                            />
                                                        }
                                                    />
                                                    <TooltipContent>
                                                        <span className="font-sans font-semibold text-xs">{entry.name}</span>
                                                        {owner ? <span className="ms-1.5 font-sans text-xs opacity-70">{owner}</span> : null}
                                                        {placed ? <span className="ms-1.5 font-mono text-[10px] uppercase tracking-wider opacity-70">{t("edit.pool.placed")}</span> : null}
                                                    </TooltipContent>
                                                </Tooltip>
                                            </li>
                                        );
                                    })}
                                </ul>
                                {pageFooter}
                            </ScrollArea>
                        )}
                    </div>

                    <DialogFooter className="justify-between border-border border-t px-6 py-3 sm:justify-between">
                        <Button type="button" variant="ghost" size="sm" onClick={clearFilters} disabled={!hasFilters}>
                            <FilterXIcon />
                            {t("edit.pool.clearFilters")}
                        </Button>
                        <DialogClose render={<Button type="button" />}>{t("edit.pool.done")}</DialogClose>
                    </DialogFooter>
                </DialogPopup>
            </Dialog>
        </TooltipProvider>
    );
}

/**
 * The pool's search, facet selections and hide-placed switch, and the
 * catalogue they leave, in the kind's pool order.
 */
function usePoolFilters(all: ITierEntity[], config: IPoolKind, placedKeys: Set<string>) {
    const [query, setQuery] = useState("");
    const [selected, setSelected] = useState<Record<string, string[]>>({});
    const [hideUsed, setHideUsed] = useState(true);

    const filtered = useMemo(() => {
        const q = compactForSearch(query);
        const matched = all.filter((entity) => {
            for (const facet of config.facets) {
                const want = selected[facet.id];
                if (want && want.length > 0 && !want.includes(facet.valueOf(entity) ?? "")) return false;
            }
            if (hideUsed && placedKeys.has(entity.key)) return false;
            if (q.length === 0) return true;
            return config.searchTexts(entity).some((text) => Boolean(text) && compactForSearch(text ?? "").includes(q));
        });
        return config.compare ? matched.sort(config.compare) : matched;
    }, [all, config, query, selected, hideUsed, placedKeys]);

    const setFacet = useCallback((id: string, values: string[]) => setSelected((prev) => ({ ...prev, [id]: values })), []);
    const clearFilters = useCallback(() => {
        setQuery("");
        setSelected({});
        setHideUsed(true);
    }, []);
    const anySelected = Object.values(selected).some((v) => v.length > 0);
    const hasFilters = query.length > 0 || anySelected || !hideUsed;

    return { query, setQuery, selected, setFacet, hideUsed, setHideUsed, filtered, hasFilters, clearFilters };
}

/**
 * The slice of a long grid rendered so far. A new `filterKey` (search or
 * filter) starts the pages over; placing a tile does not, so the grid never
 * jumps back under a scrolling editor.
 */
function usePages(filtered: ITierEntity[], filterKey: string) {
    const [limit, setLimit] = useState(PAGE);
    // biome-ignore lint/correctness/useExhaustiveDependencies: `filterKey` is the trigger, not a value read inside
    useEffect(() => setLimit(PAGE), [filterKey]);
    const paged = filtered.length > PAGE_FROM;
    const visible = paged ? filtered.slice(0, limit) : filtered;
    const showMore = useCallback(() => setLimit((n) => n + PAGE), []);
    return { paged, visible, showMore };
}

function PoolSearch({ label, value, onChange, autoFocus }: { label: string; value: string; onChange: (value: string) => void; autoFocus?: boolean }) {
    const t: TypedT<typeof messages> = useT("tierLists");
    return (
        <Field>
            <FieldLabel className="sr-only">{label}</FieldLabel>
            <InputGroup>
                <InputGroupAddon>
                    <SearchIcon aria-hidden="true" />
                </InputGroupAddon>
                <InputGroupInput value={value} onChange={(e) => onChange((e.target as HTMLInputElement).value)} placeholder={t("edit.pool.search.placeholder")} type="search" aria-label={label} autoFocus={autoFocus} />
            </InputGroup>
        </Field>
    );
}

/** What the grid shows instead of tiles: the catalogue loading, failing, or nothing matching the filters. */
function PoolEmptyState({ status, onRetry, className }: { status: CatalogueStatus; onRetry?: () => void; className: string }) {
    const t: TypedT<typeof messages> = useT("tierLists");
    if (status === "pending") {
        return (
            <output className={cn("flex items-center justify-center gap-2 font-sans text-muted-foreground text-sm", className)}>
                <Spinner />
                <span>{t("edit.pool.loading")}</span>
            </output>
        );
    }
    if (status === "error") {
        return (
            <Empty className={className}>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <TriangleAlertIcon />
                    </EmptyMedia>
                    <EmptyTitle className="text-base">{t("edit.pool.loadFailed")}</EmptyTitle>
                </EmptyHeader>
                {onRetry && (
                    <Button type="button" variant="outline" size="sm" onClick={onRetry}>
                        <RotateCwIcon />
                        {t("edit.pool.retry")}
                    </Button>
                )}
            </Empty>
        );
    }
    return (
        <Empty className={className}>
            <EmptyHeader>
                <EmptyMedia variant="icon">
                    <SearchIcon />
                </EmptyMedia>
                <EmptyTitle className="text-base">{t("edit.pool.emptyTitle")}</EmptyTitle>
                <EmptyDescription>{t("edit.pool.emptyBody")}</EmptyDescription>
            </EmptyHeader>
        </Empty>
    );
}

const EDGE_FADE_PX = 24;

/**
 * Fades whichever edge of a horizontal scroller has chips hidden past it. The
 * scrollbar is hidden, so without this a cut-off row looks complete.
 */
function useEdgeFade<T extends HTMLElement>() {
    const ref = useRef<T>(null);
    const [edges, setEdges] = useState({ start: false, end: false });
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const update = () => {
            const start = el.scrollLeft > 1;
            const end = el.scrollLeft + el.clientWidth < el.scrollWidth - 1;
            setEdges((prev) => (prev.start === start && prev.end === end ? prev : { start, end }));
        };
        update();
        el.addEventListener("scroll", update, { passive: true });
        // The row's own box stays the same width when its chips change, so watch the content too.
        const observer = new ResizeObserver(update);
        observer.observe(el);
        if (el.firstElementChild) observer.observe(el.firstElementChild);
        return () => {
            el.removeEventListener("scroll", update);
            observer.disconnect();
        };
    }, []);
    if (!edges.start && !edges.end) return { ref, style: undefined };
    const mask = `linear-gradient(to right, ${edges.start ? "transparent" : "black"}, black ${EDGE_FADE_PX}px, black calc(100% - ${EDGE_FADE_PX}px), ${edges.end ? "transparent" : "black"})`;
    return { ref, style: { maskImage: mask, WebkitMaskImage: mask } };
}

/** One facet's row of toggles in the pool dialog. */
function FacetFilter({ facet, value, onChange }: { facet: IPoolFacet; value: string[]; onChange: (next: string[]) => void }) {
    const fade = useEdgeFade<HTMLDivElement>();
    return (
        // min-w-0: a long facet (24 skin brands) has to shrink to the dialog so its row scrolls instead of overflowing.
        <Field className="min-w-0 max-w-full gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <FieldLabel className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{facet.label}</FieldLabel>
            <div ref={fade.ref} style={fade.style} className="-mx-1 flex min-w-0 overflow-x-auto px-1 [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
                <ToggleGroup value={value} onValueChange={(v) => onChange(v as string[])} aria-label={facet.groupLabel} multiple variant="outline" size="sm" className="flex-nowrap">
                    {facet.options.map((option) =>
                        facet.variant === "icon" ? (
                            <Tooltip key={option.value}>
                                <TooltipTrigger
                                    render={
                                        <ToggleGroupItem value={option.value} aria-label={option.ariaLabel ?? option.label} className="shrink-0 px-1.5 [&:not([data-pressed])>img]:opacity-40">
                                            {option.icon}
                                        </ToggleGroupItem>
                                    }
                                />
                                <TooltipContent>{option.label}</TooltipContent>
                            </Tooltip>
                        ) : (
                            <ToggleGroupItem key={option.value} value={option.value} aria-label={option.ariaLabel} className={cn("shrink-0 [&:not([data-pressed])]:opacity-55", facet.variant === "mono" && "font-mono tabular-nums")}>
                                {option.label}
                            </ToggleGroupItem>
                        ),
                    )}
                </ToggleGroup>
            </div>
        </Field>
    );
}

/**
 * Calls `onVisible` when its content is in (or near) view: the next page of a
 * long pool grid. Observes afresh after every page (`shown`), because an
 * observer only reports changes and a tall viewport can keep the sentinel in
 * view across a page.
 */
function LoadMore({ onVisible, shown, children }: { onVisible: () => void; shown: number; children: ReactNode }) {
    const ref = useRef<HTMLDivElement | null>(null);
    // biome-ignore lint/correctness/useExhaustiveDependencies: `shown` re-arms the observer after each page; it is a trigger, not a value read inside
    useEffect(() => {
        const el = ref.current;
        if (!el || typeof IntersectionObserver === "undefined") return;
        const io = new IntersectionObserver(
            (entries) => {
                if (entries.some((e) => e.isIntersecting)) onVisible();
            },
            { rootMargin: LOAD_AHEAD_MARGIN },
        );
        io.observe(el);
        return () => io.disconnect();
    }, [onVisible, shown]);
    return (
        <div ref={ref} aria-live="polite">
            {children}
        </div>
    );
}
