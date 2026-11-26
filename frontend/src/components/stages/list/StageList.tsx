import { useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronRight, Info, Lock, Maximize2, Skull } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { SearchField } from "#/components/SearchField";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/ui/page-header";
import { useTabStripScroll } from "#/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useWindowVirtualRows } from "#/hooks/use-window-virtual-rows";
import { stageIndexQueryOptions } from "#/lib/api/stages";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { STAGE_GROUP_LABEL_KEY, type StageGroupKey } from "#/lib/registry/stage-groups";
import { cn } from "#/lib/utils";
import { buildStageTree, coverImage, filterTree, type IEventVM, type IGroupVM, type IStageCardVM } from "./impl/derive";
import { EventDetailDialog } from "./impl/EventDetailDialog";
import { PreviewFallback, STAGE_GROUP_ICON, zoneCodeOf } from "./impl/PreviewFallback";
import { StagePreview } from "./impl/StagePreview";
import type { messages } from "./StageList.messages";

export function StageList() {
    const t: TypedT<typeof messages> = useT("stages");
    const tNav = useT("nav");
    const f = useFormatters();
    const { data: entries } = useSuspenseQuery(stageIndexQueryOptions(useGamedataServer()));
    const tree = useMemo(() => buildStageTree(entries), [entries]);

    const [q, setQ] = useState("");
    const [filter, setFilter] = useState<StageGroupKey | "all">("all");
    const chipStripRef = useRef<HTMLDivElement>(null);
    const chipsOverflowing = useTabStripScroll(chipStripRef);
    const [open, setOpen] = useState<Record<string, boolean>>(() => (tree.featured ? { [tree.featured.zoneId]: true } : {}));
    const [detailEvent, setDetailEvent] = useState<IEventVM | null>(null);

    const searching = q.trim().length > 0;
    const visibleGroups = useMemo(() => filterTree(tree, q, filter), [tree, q, filter]);
    const visibleEvents = useMemo(() => visibleGroups.flatMap((g) => g.events), [visibleGroups]);
    const visibleStageCount = useMemo(() => visibleEvents.reduce((n, e) => n + e.stageCount, 0), [visibleEvents]);

    // Sections (categories) default open and remember what the user folded.
    const [closedSections, setClosedSections] = useState<Record<string, boolean>>({});
    // Zone the virtualized list should scroll into view once its row is mounted.
    const [scrollZone, setScrollZone] = useState<string | null>(null);
    const toggleSection = (key: StageGroupKey) => setClosedSections((s) => ({ ...s, [key]: !s[key] }));
    const toggle = (zoneId: string) => setOpen((s) => ({ ...s, [zoneId]: !s[zoneId] }));
    const expandAll = () => {
        setOpen(Object.fromEntries(visibleEvents.map((e) => [e.zoneId, true])));
        setClosedSections({});
    };
    const collapseAll = () => setOpen({});

    const browseEvent = (ev: IEventVM) => {
        setOpen((s) => ({ ...s, [ev.zoneId]: true }));
        setClosedSections((s) => ({ ...s, [ev.group]: false }));
        setDetailEvent(null);
        setScrollZone(ev.zoneId);
    };

    const empty = visibleGroups.length === 0;

    return (
        <div className="page-shell [--page-max:1200px]">
            <PageHeader
                className="mb-6 border-border border-b pb-6 sm:mb-7"
                breadcrumbLabel={t("list.breadcrumb")}
                breadcrumb={[t("list.breadcrumb.collection"), t("list.breadcrumb.stages")]}
                title={t("list.breadcrumb.stages")}
                description={t("list.blurb")}
                actions={<p className="m-0 font-sans text-[13px] text-muted-foreground tabular-nums">{t("list.counts", { stages: f.number(tree.totalStages), zones: f.number(tree.totalZones), categories: tree.groups.length })}</p>}
            />

            {!searching && tree.featured && (filter === "all" || filter === tree.featured.group) && (
                <FeaturedHero
                    event={tree.featured}
                    onBrowse={() => {
                        const featured = tree.featured;
                        if (featured) browseEvent(featured);
                    }}
                    onDetails={() => setDetailEvent(tree.featured)}
                />
            )}

            <div className="mb-3 flex flex-wrap items-center gap-x-3 gap-y-2.5">
                <SearchField className="min-w-0 flex-1 basis-full sm:max-w-100 sm:basis-auto" value={q} onChange={setQ} placeholder={t("list.search.placeholder")} label={t("list.search.aria")} />
                <span className="font-mono text-[11px] text-muted-foreground">{searching ? t("list.count.matches", { count: visibleStageCount }) : t("list.count.zones", { count: visibleEvents.length })}</span>
                <div className="ml-auto flex gap-1">
                    <Button onClick={collapseAll} size="xs" variant="ghost">
                        {t("list.collapse")}
                    </Button>
                    <Button onClick={expandAll} size="xs" variant="ghost">
                        {t("list.expandAll")}
                    </Button>
                </div>
            </div>

            <ToggleGroup
                aria-label={t("list.filter.label")}
                className={cn("mb-8 w-auto max-w-full flex-nowrap gap-1.5 overflow-x-auto overflow-y-hidden pb-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden", chipsOverflowing && "scroll-shadows-x")}
                onValueChange={(next) => setFilter((next[0] as StageGroupKey | "all" | undefined) ?? "all")}
                ref={chipStripRef}
                size="sm"
                value={[filter]}
                variant="outline"
            >
                <ToggleGroupItem className="flex-none gap-1.5 px-3" value="all">
                    {t("list.filter.all")}
                    <Badge className="font-mono tabular-nums" size="sm" variant="secondary">
                        {tree.totalZones}
                    </Badge>
                </ToggleGroupItem>
                {tree.groups.map((g) => (
                    <ToggleGroupItem className="flex-none gap-1.5 px-3" key={g.key} value={g.key}>
                        <span aria-hidden className="size-2 shrink-0 rounded-full" style={{ background: g.tone }} />
                        {tNav(STAGE_GROUP_LABEL_KEY[g.key])}
                        <Badge className="font-mono tabular-nums" size="sm" variant="secondary">
                            {g.zoneCount}
                        </Badge>
                    </ToggleGroupItem>
                ))}
            </ToggleGroup>

            {empty && <div className="rounded-2xl border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("list.empty")}</div>}

            {/* Timeline spine: decoration only above sm, so phones get the full width back. */}
            <div className="relative sm:pl-6.5">
                <div className="absolute top-2 bottom-2.5 left-0.75 hidden w-px sm:block" style={{ background: "linear-gradient(180deg,var(--border),color-mix(in oklch,var(--border) 25%,transparent))" }} />
                <VirtualStageTree groups={visibleGroups} searching={searching} open={open} closedSections={closedSections} onToggleSection={toggleSection} onToggleZone={toggle} onDetails={setDetailEvent} scrollZone={scrollZone} onScrolled={() => setScrollZone(null)} />
            </div>

            <EventDetailDialog event={detailEvent} onClose={() => setDetailEvent(null)} onBrowse={browseEvent} />
        </div>
    );
}

const STAGE_VIRTUAL_OVERSCAN = 6;

/**
 * The visible tree flattened into one array so a single window virtualizer can
 * window it. A `group` header, an `event` accordion header, or one `stages`
 * grid-row (a chunk of stage cards). The group's rounded card, its per-event
 * separators, and the stage panel's border/background are reconstructed across
 * the flattened rows so the output matches the original nested markup.
 */
type FlatRow =
    | { key: string; kind: "group"; group: IGroupVM; first: boolean }
    | { key: string; kind: "event"; group: IGroupVM; event: IEventVM; open: boolean; containerFirst: boolean; containerLast: boolean }
    | { key: string; kind: "stages"; group: IGroupVM; event: IEventVM; cards: IStageCardVM[]; first: boolean; last: boolean; containerLast: boolean };

function buildFlatRows(groups: IGroupVM[], searching: boolean, open: Record<string, boolean>, closedSections: Record<string, boolean>, cols: number): FlatRow[] {
    const rows: FlatRow[] = [];
    const sectionOpen = (key: StageGroupKey) => searching || !closedSections[key];
    const zoneOpen = (zoneId: string) => searching || !!open[zoneId];
    groups.forEach((group, gi) => {
        rows.push({ key: `g-${group.key}`, kind: "group", group, first: gi === 0 });
        if (!sectionOpen(group.key)) return;
        group.events.forEach((event, ei) => {
            const isLastEvent = ei === group.events.length - 1;
            if (zoneOpen(event.zoneId)) {
                const chunks: IStageCardVM[][] = [];
                for (let i = 0; i < event.stages.length; i += cols) chunks.push(event.stages.slice(i, i + cols));
                const headerLast = chunks.length === 0;
                rows.push({ key: `e-${event.zoneId}`, kind: "event", group, event, open: true, containerFirst: ei === 0, containerLast: headerLast && isLastEvent });
                chunks.forEach((cards, ci) => {
                    const last = ci === chunks.length - 1;
                    rows.push({ key: `s-${event.zoneId}-${ci}`, kind: "stages", group, event, cards, first: ci === 0, last, containerLast: last && isLastEvent });
                });
            } else {
                rows.push({ key: `e-${event.zoneId}`, kind: "event", group, event, open: false, containerFirst: ei === 0, containerLast: isLastEvent });
            }
        });
    });
    return rows;
}

/**
 * A row's place inside its group's panel. The panel itself is one element drawn behind the rows (see
 * `GroupPanels`): per-row side borders read as separate square strips with hairline gaps where the
 * measured row heights rounded. Rows sit 1px inside the panel border and only clip their own tint to its corners.
 */
function containerRow(row: { containerFirst?: boolean; containerLast: boolean }): string {
    return cn("mx-px", row.containerFirst && "overflow-hidden rounded-t-[calc(var(--radius-2xl)-1px)]", row.containerLast && "overflow-hidden rounded-b-[calc(var(--radius-2xl)-1px)]");
}

/** One `rounded-2xl border bg-card` panel per open group, spanning its first zone row to its last. */
function GroupPanels({ rows, measurements, scrollMargin }: { rows: FlatRow[]; measurements: ReadonlyArray<{ start: number; end: number }>; scrollMargin: number }) {
    const spans: Array<{ key: string; top: number; height: number }> = [];
    let first = -1;
    rows.forEach((row, i) => {
        if (row.kind === "event" && row.containerFirst) first = i;
        if (row.kind !== "group" && row.containerLast && first >= 0) {
            const a = measurements[first];
            const b = measurements[i];
            if (a && b) spans.push({ key: row.group.key, top: a.start - scrollMargin - 1, height: b.end - a.start + 2 });
            first = -1;
        }
    });
    return (
        <>
            {spans.map((span) => (
                <div key={span.key} aria-hidden="true" className="pointer-events-none absolute inset-x-0 rounded-2xl border border-border bg-card" style={{ top: span.top, height: span.height }} />
            ))}
        </>
    );
}

interface IVirtualStageTreeProps {
    groups: IGroupVM[];
    searching: boolean;
    open: Record<string, boolean>;
    closedSections: Record<string, boolean>;
    onToggleSection: (key: StageGroupKey) => void;
    onToggleZone: (zoneId: string) => void;
    onDetails: (ev: IEventVM) => void;
    scrollZone: string | null;
    onScrolled: () => void;
}

/** The viewport width the server lays the stage tree out for. */
const SSR_VIEWPORT_WIDTH = 1280;

function VirtualStageTree({ groups, searching, open, closedSections, onToggleSection, onToggleZone, onDetails, scrollZone, onScrolled }: IVirtualStageTreeProps) {
    // The server renders at 1280 and the first client render must match it: reading
    // window.innerWidth here gave a phone the small-screen gaps on hydration
    // (17764px against the server's 17818px at 375). The real width lands in the effect.
    const [viewportWidth, setViewportWidth] = useState(SSR_VIEWPORT_WIDTH);
    useEffect(() => {
        const onResize = () => setViewportWidth(window.innerWidth);
        onResize();
        window.addEventListener("resize", onResize, { passive: true });
        return () => window.removeEventListener("resize", onResize);
    }, []);

    // Mirror the responsive stage grid (auto-fill minmax columns) so chunked rows size cards identically.
    const isSm = viewportWidth >= 640;
    const gridPadX = isSm ? 16 : 12; // px-4 / px-3
    const minCard = isSm ? 154 : 126;
    const gridGap = isSm ? 12 : 10; // gap-3 / gap-2.5
    const columnsFor = (w: number) => {
        const inner = w - gridPadX * 2 - 2;
        if (inner <= 0) return 1;
        return Math.max(1, Math.floor((inner + gridGap) / (minCard + gridGap)));
    };
    const cardHeightFor = (w: number) => {
        const cols = columnsFor(w);
        const inner = w - gridPadX * 2 - 2;
        const cardW = inner > 0 ? (inner - (cols - 1) * gridGap) / cols : 140;
        return (cardW * 10) / 16 + 56;
    };

    const { parentRef, width, scrollMargin, rows, virtualizer } = useWindowVirtualRows<FlatRow>({
        buildRows: (w) => buildFlatRows(groups, searching, open, closedSections, columnsFor(w)),
        rowDeps: [groups, searching, open, closedSections, viewportWidth],
        estimateSize: (row, _index, w) => {
            if (!row) return 80;
            if (row.kind === "group") return row.first ? 44 : 80;
            if (row.kind === "event") return 72;
            const top = row.first ? 20 : gridGap;
            const bottom = row.last ? (isSm ? 18 : 16) : 0;
            return Math.round(top + cardHeightFor(w) + bottom);
        },
        overscan: STAGE_VIRTUAL_OVERSCAN,
        measureDeps: [viewportWidth],
        getRowKey: (row) => row.key,
    });

    const cols = columnsFor(width);

    useEffect(() => {
        if (!scrollZone) return;
        const index = rows.findIndex((r) => r.kind === "event" && r.event.zoneId === scrollZone);
        if (index >= 0) {
            const smooth = typeof window !== "undefined" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
            virtualizer.scrollToIndex(index, { align: "center", behavior: smooth ? "smooth" : "auto" });
        }
        onScrolled();
    }, [scrollZone, rows, virtualizer, onScrolled]);

    const virtualItems = virtualizer.getVirtualItems();

    return (
        <div ref={parentRef} style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
            <GroupPanels rows={rows} measurements={virtualizer.measurementsCache} scrollMargin={scrollMargin} />
            {virtualItems.map((vi) => {
                const row = rows[vi.index];
                if (!row) return null;
                return (
                    <div data-index={vi.index} key={row.key} ref={virtualizer.measureElement} style={{ left: 0, position: "absolute", top: 0, transform: `translateY(${vi.start - scrollMargin}px)`, width: "100%" }}>
                        {row.kind === "group" ? (
                            <GroupHeaderRow row={row} open={searching || !closedSections[row.group.key]} onToggle={() => onToggleSection(row.group.key)} />
                        ) : row.kind === "event" ? (
                            <EventHeaderRow row={row} onToggle={() => onToggleZone(row.event.zoneId)} onDetails={() => onDetails(row.event)} />
                        ) : (
                            <StagesRow row={row} cols={cols} />
                        )}
                    </div>
                );
            })}
        </div>
    );
}

function GroupHeaderRow({ row, open, onToggle }: { row: Extract<FlatRow, { kind: "group" }>; open: boolean; onToggle: () => void }) {
    const t: TypedT<typeof messages> = useT("stages");
    const tNav = useT("nav");
    const { group, first } = row;
    const Icon = STAGE_GROUP_ICON[group.key];
    return (
        <section style={{ scrollMarginTop: 74 }} className={cn("relative", !first && "pt-7 sm:pt-8")}>
            <div className="relative pb-2 sm:pb-3.5">
                <span className="absolute top-2.5 -left-5.75 hidden h-1.75 w-1.75 rotate-45 sm:block" style={{ background: group.tone, boxShadow: "0 0 0 4px var(--background)" }} />
                <h2 className="m-0">
                    <button type="button" onClick={onToggle} aria-expanded={open} aria-controls={`section-${group.key}`} className="group/section flex w-full cursor-pointer flex-wrap items-baseline gap-x-3 gap-y-0.5 rounded-md py-1 text-left ring-inset focus-visible:ring-2 focus-visible:ring-ring">
                        <Icon className="h-3.75 w-3.75 flex-none self-center" style={{ color: group.tone }} aria-hidden="true" />
                        <span className="font-bold font-sans text-[17px] text-foreground tracking-[-0.02em]">{tNav(STAGE_GROUP_LABEL_KEY[group.key])}</span>
                        <span className="whitespace-nowrap font-mono text-[10px] text-muted-foreground tracking-widest">{t("list.group.counts", { zones: group.zoneCount, stages: group.stageCount })}</span>
                        <span className="h-px flex-1 bg-border" />
                        <ChevronDown className="h-3.75 w-3.75 flex-none self-center text-muted-foreground transition-[transform,color] duration-200 group-hover/section:text-foreground" style={{ transform: open ? "rotate(180deg)" : "rotate(0deg)" }} aria-hidden="true" />
                    </button>
                </h2>
            </div>
        </section>
    );
}

function EventHeaderRow({ row, onToggle, onDetails }: { row: Extract<FlatRow, { kind: "event" }>; onToggle: () => void; onDetails: () => void }) {
    const t: TypedT<typeof messages> = useT("stages");
    const { event, open, group } = row;
    const cover = coverImage(event);
    return (
        <div className={cn(containerRow(row), !row.containerFirst && "border-border border-t")} style={{ ["--rowtone" as string]: event.tone }} {...(row.containerFirst ? { id: `section-${group.key}` } : {})}>
            <div id={`zone-${event.zoneId}`} className="msv-row relative flex items-center gap-3 px-3.5 py-3.5 sm:gap-4 sm:px-4.5" style={{ background: open ? `color-mix(in oklch, ${event.tone} 7%, transparent)` : "transparent" }}>
                {open && <span className="pointer-events-none absolute top-0 bottom-0 left-0 z-1 w-0.5 rounded-r" style={{ background: event.tone }} />}

                <button type="button" onClick={onToggle} aria-expanded={open} aria-label={open ? t("list.zone.collapse", { title: event.title }) : t("list.zone.expand", { title: event.title })} className="absolute inset-0 cursor-pointer rounded-none ring-inset focus-visible:ring-2 focus-visible:ring-ring" />

                <button
                    type="button"
                    onClick={onDetails}
                    aria-haspopup="dialog"
                    aria-label={t("list.zone.about", { title: event.title })}
                    className="group relative z-1 h-10 pointer-coarse:h-11 w-16 flex-none cursor-zoom-in overflow-hidden rounded-md border border-border ring-inset focus-visible:ring-2 focus-visible:ring-ring"
                >
                    <PreviewFallback tone={event.tone} group={event.group} label={zoneCodeOf(event.codeRange)} iconClassName="h-3.5 w-3.5" />
                    {cover && <StagePreview src={cover} />}
                    <span className="absolute inset-0 flex items-center justify-center bg-black/45 opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-visible:opacity-100">
                        <Maximize2 className="h-3.5 w-3.5 text-white" aria-hidden="true" />
                    </span>
                </button>

                <div className="pointer-events-none relative z-1 min-w-0 flex-1">
                    <div className="flex items-center gap-2.5">
                        <span className="truncate font-[650] font-sans text-[15px] text-foreground leading-[1.15] tracking-[-0.01em]">{event.title}</span>
                        {event.kicker && <span className="whitespace-nowrap font-mono text-[10px] text-muted-foreground">{event.kicker}</span>}
                    </div>
                    {event.codeRange && <div className="mt-0.5 font-mono text-[10px] text-muted-foreground tracking-[0.04em]">{event.codeRange}</div>}
                </div>

                <span className="pointer-events-none relative z-1 hidden whitespace-nowrap font-mono text-[10.5px] text-muted-foreground sm:inline">
                    {t("list.zone.ops", { count: event.stageCount })}
                    {event.originitePrime > 0 ? ` · ${t("list.zone.originitePrime", { count: event.originitePrime })}` : ""}
                    {event.bossCount > 0 ? ` · ${t("list.zone.boss", { count: event.bossCount })}` : ""}
                </span>
                <ChevronDown className="pointer-events-none relative z-1 h-3.75 w-3.75 flex-none text-muted-foreground transition-transform duration-200" style={{ transform: open ? "rotate(180deg)" : "rotate(0deg)" }} aria-hidden="true" />
            </div>
        </div>
    );
}

function StagesRow({ row, cols }: { row: Extract<FlatRow, { kind: "stages" }>; cols: number }) {
    const { event } = row;
    return (
        <div className={containerRow(row)} style={{ ["--rowtone" as string]: event.tone }}>
            <div className={cn("bg-background/40 px-3 sm:px-4", row.first ? "border-border border-t pt-5" : "pt-2.5 sm:pt-3", row.last && "pb-4 sm:pb-4.5")}>
                <div className="grid gap-2.5 sm:gap-3" style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}>
                    {row.cards.map((st) => (
                        <StageCard key={st.stageId} card={st} tone={event.tone} group={event.group} />
                    ))}
                </div>
            </div>
        </div>
    );
}

function StageCardInner({ card, tone, group }: { card: IStageCardVM; tone: string; group: StageGroupKey }) {
    const t: TypedT<typeof messages> = useT("stages");
    return (
        <>
            <div className="relative aspect-16/10 w-full flex-none overflow-hidden">
                <PreviewFallback tone={tone} group={group} />
                {card.preview && (
                    <>
                        <StagePreview src={card.preview} />
                        <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.9) 0%, rgba(7,9,12,0.15) 46%, rgba(7,9,12,0) 66%)" }} />
                    </>
                )}
                {card.badge && (
                    <span
                        className="absolute bottom-1.75 left-1.75 z-2 inline-flex h-4.5 max-w-[calc(100%-14px)] items-center overflow-hidden text-ellipsis whitespace-nowrap rounded-[5px] border px-1.75 font-bold font-mono text-[10px] text-white leading-none"
                        style={{ borderColor: `color-mix(in oklch, ${tone} 50%, transparent)`, background: "rgba(0,0,0,0.6)" }}
                    >
                        {card.badge}
                    </span>
                )}
                {card.boss && (
                    <span
                        className="absolute top-1.75 right-1.75 z-2 inline-flex h-4.5 items-center gap-1 rounded-full px-1.75 font-bold font-mono text-[10px] text-white"
                        style={{ background: "color-mix(in oklch, var(--enemy-boss) 88%, black)", boxShadow: "0 0 10px color-mix(in oklch, var(--enemy-boss) 60%, transparent)" }}
                    >
                        <Skull className="h-2.5 w-2.5" /> {t("list.card.boss")}
                    </span>
                )}
            </div>
            <div className="flex flex-col gap-1.5 px-2.5 pt-2 pb-2.5">
                <span className="line-clamp-2 min-h-8 font-sans font-semibold text-[12.5px] text-foreground leading-tight">{card.title}</span>
                <div className="flex items-center justify-between gap-1.5">
                    <span className="font-mono text-[10px] text-muted-foreground">{card.apCost > 0 ? t("list.card.sanity", { cost: card.apCost }) : t("list.card.free")}</span>
                </div>
            </div>
        </>
    );
}

function StageCard({ card, tone, group }: { card: IStageCardVM; tone: string; group: StageGroupKey }) {
    const cardStyle = { ["--rowtone" as string]: tone };
    // Variant links sit over the card's footer as siblings of the card link, never inside it.
    const variants = card.variants.length > 0 && <StageVariantLinks card={card} />;

    if (!card.canView) {
        return (
            <div className="relative">
                <div className="flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left" style={{ ...cardStyle, cursor: "default" }}>
                    <StageCardInner card={card} tone={tone} group={group} />
                </div>
                {variants}
            </div>
        );
    }
    return (
        <div className="msv-card relative">
            <Link to="/stages/$stageId" params={{ stageId: card.stageId }} className="msv-stage flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left transition-[transform,border-color,box-shadow] duration-150" style={cardStyle}>
                <StageCardInner card={card} tone={tone} group={group} />
            </Link>
            {variants}
        </div>
    );
}

function StageVariantLinks({ card }: { card: IStageCardVM }) {
    const t: TypedT<typeof messages> = useT("stages");
    return (
        <div className="msv-variants absolute right-2 bottom-2 z-2 flex items-center gap-1 transition-transform duration-150">
            {card.variants.map((v) => {
                const text = v.kind === "cm" ? t("list.card.cm") : t("list.card.adverse");
                const label = v.kind === "cm" ? t("list.card.cm.label", { title: card.title }) : t("list.card.adverse.label", { title: card.title });
                const chip = "inline-flex h-4.5 items-center rounded-[5px] border border-border px-1.5 font-mono font-semibold text-[10px] leading-none";
                if (!v.canView) {
                    return (
                        <span key={v.stageId} className={cn(chip, "gap-0.5 bg-card text-subtle-foreground")} title={t("list.card.variant.unavailable", { label })}>
                            <Lock aria-hidden="true" className="size-2.5" />
                            <span aria-hidden="true">{text}</span>
                            <span className="sr-only">{t("list.card.variant.unavailable", { label })}</span>
                        </span>
                    );
                }
                return (
                    <Link
                        key={v.stageId}
                        to="/stages/$stageId"
                        params={{ stageId: v.stageId }}
                        aria-label={label}
                        className={cn(
                            chip,
                            "pointer-coarse:after:-translate-1/2 relative bg-card text-muted-foreground transition-colors pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 hover:border-(--rowtone) hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring",
                        )}
                    >
                        {text}
                    </Link>
                );
            })}
        </div>
    );
}

function FeaturedHero({ event, onBrowse, onDetails }: { event: IEventVM; onBrowse: () => void; onDetails: () => void }) {
    const t: TypedT<typeof messages> = useT("stages");
    const tNav = useT("nav");
    const cover = coverImage(event);
    return (
        <div className="relative mb-7 h-66 overflow-hidden rounded-2xl border border-border bg-card sm:mb-8 sm:h-60">
            <PreviewFallback tone={event.tone} group={event.group} iconClassName="h-8 w-8" />
            {cover && <StagePreview src={cover} />}
            {/* Legibility + brand wash: side sweep for wide screens, bottom anchor so text survives bright art at any width. */}
            <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(90deg, rgba(7,9,12,0.86) 0%, rgba(7,9,12,0.55) 42%, rgba(7,9,12,0.15) 100%)" }} />
            <div className="pointer-events-none absolute inset-0 max-sm:hidden" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.85) 0%, rgba(7,9,12,0.35) 45%, rgba(7,9,12,0) 70%)" }} />
            <div className="pointer-events-none absolute inset-0" style={{ background: "radial-gradient(90% 130% at 90% 0%, color-mix(in oklch, var(--primary) 26%, transparent), transparent 55%)" }} />
            {/* Below sm the title wraps up into the art and runs full width, past the side sweep. Every point a title or meta line can
                reach (the lower 75%) keeps >= 0.68 of the scrim: over pure-white art white text then measures >= 7.16:1, white/75 >= 4.89:1. */}
            <div className="pointer-events-none absolute inset-0 sm:hidden" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.82) 0%, rgba(7,9,12,0.78) 62%, rgba(7,9,12,0.5) 100%)" }} />

            <div className="absolute inset-0 flex flex-col justify-between p-4.5 sm:p-7">
                <div className="flex items-center gap-2.5">
                    <span className="inline-flex h-5.5 items-center rounded-full border px-2.5 font-mono font-semibold text-[10px] text-white" style={{ borderColor: `color-mix(in oklch, ${event.tone} 55%, transparent)`, background: `color-mix(in oklch, ${event.tone} 16%, rgba(0,0,0,0.35))` }}>
                        {tNav(STAGE_GROUP_LABEL_KEY[event.group])}
                    </span>
                    {event.kicker && <span className="kicker text-white/75">{event.kicker}</span>}
                </div>
                <div>
                    <div className="text-balance font-bold font-sans text-[25px] text-white leading-[1.02] tracking-[-0.03em] sm:max-w-[80%] sm:text-[34px]" style={{ textShadow: "0 2px 24px rgba(0,0,0,0.55)" }}>
                        {event.title}
                    </div>
                    <div className="mt-3 flex flex-wrap items-center gap-x-3.5 gap-y-2 sm:mt-3.5">
                        {event.codeRange && <span className="font-mono text-[11px] text-white/75">{event.codeRange}</span>}
                        {event.codeRange && <span className="h-3 w-px bg-white/25" />}
                        <span className="font-mono text-[11px] text-white/75">
                            {event.stageCount} {t("list.hero.operations", { count: event.stageCount })}
                        </span>
                        {event.bossCount > 0 && (
                            <>
                                <span className="h-3 w-px bg-white/25" />
                                <span className="font-mono text-[11px] text-white">
                                    {event.bossCount} {t("list.hero.bosses", { count: event.bossCount })}
                                </span>
                            </>
                        )}
                        <button
                            type="button"
                            onClick={onBrowse}
                            className="relative inline-flex h-10 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-lg bg-primary px-4 font-sans font-semibold text-[12.5px] text-primary-foreground shadow-sm transition-transform pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:-translate-y-px sm:ml-1 sm:h-8.5"
                        >
                            {t("list.hero.browse")}
                            <ChevronRight className="h-3.25 w-3.25" aria-hidden="true" />
                        </button>
                        <button
                            type="button"
                            onClick={onDetails}
                            aria-haspopup="dialog"
                            className="relative inline-flex h-10 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-lg border border-white/25 bg-white/10 px-3.5 font-sans font-semibold text-[12.5px] text-white backdrop-blur-sm transition-colors pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:bg-white/18 sm:h-8.5"
                        >
                            <Info className="h-3.25 w-3.25" aria-hidden="true" />
                            {t("list.hero.details")}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
