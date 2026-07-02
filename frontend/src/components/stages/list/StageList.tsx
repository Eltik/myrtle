import { useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronRight, Info, Maximize2, Search, Skull } from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";
import { stageIndexQueryOptions } from "#/lib/api/stages";
import type { StageGroupKey } from "#/lib/registry/stage-groups";
import { buildStageTree, coverImage, filterTree, type IEventVM, type IGroupVM, type IStageCardVM } from "./impl/derive";
import { EventDetailDialog } from "./impl/EventDetailDialog";
import { PreviewFallback, STAGE_GROUP_ICON } from "./impl/PreviewFallback";
import { StagePreview } from "./impl/StagePreview";

export function StageList() {
    const { data: entries } = useSuspenseQuery(stageIndexQueryOptions());
    const tree = useMemo(() => buildStageTree(entries), [entries]);

    const [q, setQ] = useState("");
    const [filter, setFilter] = useState<StageGroupKey | "all">("all");
    const [open, setOpen] = useState<Record<string, boolean>>(() => (tree.featured ? { [tree.featured.zoneId]: true } : {}));
    const [detailEvent, setDetailEvent] = useState<IEventVM | null>(null);

    const searching = q.trim().length > 0;
    const visibleGroups = useMemo(() => filterTree(tree, q, filter), [tree, q, filter]);
    const visibleEvents = useMemo(() => visibleGroups.flatMap((g) => g.events), [visibleGroups]);
    const visibleStageCount = useMemo(() => visibleEvents.reduce((n, e) => n + e.stageCount, 0), [visibleEvents]);

    // Sections (categories) default open and remember what the user folded.
    const [closedSections, setClosedSections] = useState<Record<string, boolean>>({});
    const isSectionOpen = (key: StageGroupKey) => searching || !closedSections[key];
    const toggleSection = (key: StageGroupKey) => setClosedSections((s) => ({ ...s, [key]: !s[key] }));

    const isOpen = (zoneId: string) => searching || !!open[zoneId];
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
        requestAnimationFrame(() => {
            const smooth = !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
            document.getElementById(`zone-${ev.zoneId}`)?.scrollIntoView({ behavior: smooth ? "smooth" : "auto", block: "center" });
        });
    };

    const empty = visibleGroups.length === 0;

    return (
        <div className="relative z-1 mx-auto w-[min(1200px,calc(100%-2rem))] pb-28">
            {/* Breadcrumb */}
            <nav className="pt-7 pb-0 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label="Breadcrumb">
                <ol className="flex items-center gap-1.5">
                    <li>Collection</li>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <li className="text-foreground">Stages</li>
                </ol>
            </nav>

            <div className="mt-6 mb-6 flex flex-col gap-5 border-border border-b pb-6 sm:mb-7 lg:flex-row lg:items-end lg:justify-between lg:gap-6">
                <div>
                    <div className="mb-3 flex items-center gap-2.5">
                        <span className="h-px w-5" style={{ background: "var(--primary)" }} />
                        <span className="font-mono font-semibold text-[10.5px] text-primary uppercase tracking-[0.24em]">The Stage Record</span>
                    </div>
                    <h1 className="m-0 text-balance font-bold font-sans text-[27px] text-foreground leading-[0.98] tracking-[-0.035em] sm:text-[34px]">Every operation, catalogued.</h1>
                    <p className="mt-3 max-w-140 text-pretty font-sans text-[13px] text-muted-foreground leading-[1.55]">Browse by story arc and code - each stage opens to a faux-3D board and enemy-pathing simulator. Ordered by episode and code.</p>
                </div>
                <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1 lg:block lg:text-right">
                    <div className="font-light font-mono text-[26px] text-primary leading-none tracking-[-0.02em] lg:text-[34px]">{tree.totalStages.toLocaleString()}</div>
                    <div className="font-mono text-[10px] text-muted-foreground tracking-[0.12em] lg:mt-1.5">
                        STAGES · {tree.totalZones} ZONES · {tree.groups.length} CATEGORIES
                    </div>
                </div>
            </div>

            {/* Featured hero */}
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
                <div className="relative min-w-0 flex-1 basis-full sm:max-w-100 sm:basis-auto">
                    <Search className="pointer-events-none absolute top-1/2 left-3.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                    <input
                        value={q}
                        onChange={(e) => setQ(e.target.value)}
                        placeholder="Search event, code or stage…"
                        aria-label="Search stages"
                        className="h-10 w-full rounded-[9px] border border-border bg-secondary/50 pr-3.5 pl-9 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 sm:h-9.5"
                    />
                </div>
                <span className="font-mono text-[11px] text-muted-foreground">{searching ? `${visibleStageCount} matches` : `${visibleEvents.length} zones`}</span>
                <div className="ml-auto flex gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75">
                    <button type="button" onClick={collapseAll} className="h-8 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] text-muted-foreground transition-colors hover:text-foreground sm:h-6.5">
                        Collapse
                    </button>
                    <button type="button" onClick={expandAll} className="h-8 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] text-muted-foreground transition-colors hover:text-foreground sm:h-6.5">
                        Expand all
                    </button>
                </div>
            </div>

            {/* Filter pills */}
            <div className="msv-scroll mb-8 flex gap-1.5 overflow-x-auto pb-1.5">
                <FilterPill label="All" count={tree.totalZones} tone="var(--primary)" active={filter === "all"} onClick={() => setFilter("all")} />
                {tree.groups.map((g) => (
                    <FilterPill key={g.key} label={g.label} count={g.zoneCount} tone={g.tone} active={filter === g.key} onClick={() => setFilter((f) => (f === g.key ? "all" : g.key))} />
                ))}
            </div>

            {empty && <div className="rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">No stages match your search.</div>}

            {/* Timeline spine - decoration only above sm; phones get the full width back. */}
            <div className="relative sm:pl-6.5">
                <div className="absolute top-2 bottom-2.5 left-0.75 hidden w-px sm:block" style={{ background: "linear-gradient(180deg,var(--border),color-mix(in oklch,var(--border) 25%,transparent))" }} />
                <div className="flex flex-col gap-7 sm:gap-8">
                    {visibleGroups.map((g) => (
                        <GroupSection key={g.key} group={g} open={isSectionOpen(g.key)} onToggle={() => toggleSection(g.key)} isOpen={isOpen} toggle={toggle} onDetails={setDetailEvent} />
                    ))}
                </div>
            </div>

            <EventDetailDialog event={detailEvent} onClose={() => setDetailEvent(null)} onBrowse={browseEvent} />
        </div>
    );
}

function FilterPill({ label, count, tone, active, onClick }: { label: string; count: number; tone: string; active: boolean; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="msv-pill inline-flex h-8.5 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-full border px-3 font-sans font-semibold text-[12px] transition-colors sm:h-7.5"
            style={{
                borderColor: active ? tone : "var(--border)",
                background: active ? `color-mix(in oklch, ${tone} 15%, transparent)` : "color-mix(in oklch, var(--secondary) 45%, transparent)",
                color: active ? tone : "var(--muted-foreground)",
            }}
        >
            <span>{label}</span>
            <span
                className="rounded-full px-1.5 py-px font-mono font-semibold text-[9.5px]"
                style={{
                    background: active ? `color-mix(in oklch, ${tone} 22%, transparent)` : "color-mix(in oklch, var(--muted) 55%, transparent)",
                    color: active ? tone : "var(--muted-foreground)",
                }}
            >
                {count}
            </span>
        </button>
    );
}

function GroupSection({ group, open, onToggle, isOpen, toggle, onDetails }: { group: IGroupVM; open: boolean; onToggle: () => void; isOpen: (id: string) => boolean; toggle: (id: string) => void; onDetails: (ev: IEventVM) => void }) {
    const Icon = STAGE_GROUP_ICON[group.key];
    return (
        <section style={{ scrollMarginTop: 74 }}>
            <div className="relative mb-2 sm:mb-3.5">
                <span className="absolute top-2.5 -left-5.75 hidden h-1.75 w-1.75 rotate-45 sm:block" style={{ background: group.tone, boxShadow: "0 0 0 4px var(--background)" }} />
                <h2 className="m-0">
                    <button type="button" onClick={onToggle} aria-expanded={open} aria-controls={`section-${group.key}`} className="group/section flex w-full cursor-pointer flex-wrap items-baseline gap-x-3 gap-y-0.5 rounded-md py-1 text-left ring-inset focus-visible:ring-2 focus-visible:ring-ring/60">
                        <Icon className="h-3.75 w-3.75 flex-none self-center" style={{ color: group.tone }} aria-hidden="true" />
                        <span className="font-bold font-sans text-[17px] text-foreground tracking-[-0.02em]">{group.label}</span>
                        <span className="whitespace-nowrap font-mono text-[10px] text-muted-foreground tracking-widest">
                            {group.zoneCount} zones · {group.stageCount} stages
                        </span>
                        <span className="h-px flex-1 bg-border" />
                        <ChevronDown className="h-3.75 w-3.75 flex-none self-center text-muted-foreground transition-[transform,color] duration-200 group-hover/section:text-foreground" style={{ transform: open ? "rotate(180deg)" : "rotate(0deg)" }} aria-hidden="true" />
                    </button>
                </h2>
            </div>

            <CollapsiblePanel open={open}>
                <div id={`section-${group.key}`} className="overflow-hidden rounded-xl border border-border bg-card/40">
                    {group.events.map((ev) => (
                        <EventRow key={ev.zoneId} event={ev} open={isOpen(ev.zoneId)} onToggle={() => toggle(ev.zoneId)} onDetails={() => onDetails(ev)} />
                    ))}
                </div>
            </CollapsiblePanel>
        </section>
    );
}

/**
 * Animated expand/collapse via the grid-rows 0fr→1fr trick (no measured
 * heights). Children stay mounted while the close animation plays, then
 * unmount so collapsed rows don't keep hundreds of cards in the DOM.
 */
function CollapsiblePanel({ open, children }: { open: boolean; children: ReactNode }) {
    const [mounted, setMounted] = useState(open);
    if (open && !mounted) setMounted(true);
    return (
        <div
            className="grid transition-[grid-template-rows] duration-300 ease-out motion-reduce:transition-none"
            style={{ gridTemplateRows: open ? "1fr" : "0fr" }}
            onTransitionEnd={(e) => {
                if (e.target === e.currentTarget && !open) setMounted(false);
            }}
        >
            <div className={`min-h-0 overflow-hidden transition-opacity duration-200 motion-reduce:transition-none ${open ? "opacity-100" : "opacity-0"}`}>{mounted ? children : null}</div>
        </div>
    );
}

function EventRow({ event, open, onToggle, onDetails }: { event: IEventVM; open: boolean; onToggle: () => void; onDetails: () => void }) {
    const cover = coverImage(event);
    return (
        <div id={`zone-${event.zoneId}`} className="border-border border-b last:border-b-0" style={{ ["--rowtone" as string]: event.tone }}>
            <div className="msv-row relative flex items-center gap-3 px-3.5 py-3.5 sm:gap-4 sm:px-4.5" style={{ background: open ? `color-mix(in oklch, ${event.tone} 7%, transparent)` : "transparent" }}>
                {open && <span className="pointer-events-none absolute top-0 bottom-0 left-0 z-1 w-0.5 rounded-r" style={{ background: event.tone }} />}

                <button type="button" onClick={onToggle} aria-expanded={open} aria-label={`${open ? "Collapse" : "Expand"} ${event.title}`} className="absolute inset-0 cursor-pointer rounded-none ring-inset focus-visible:ring-2 focus-visible:ring-ring/60" />

                {/* Zone thumbnail: opens the enlarged-banner detail dialog. */}
                <button type="button" onClick={onDetails} aria-haspopup="dialog" aria-label={`About ${event.title}`} className="group relative z-1 h-10 w-16 flex-none cursor-zoom-in overflow-hidden rounded-md border border-border ring-inset focus-visible:ring-2 focus-visible:ring-ring/60">
                    <PreviewFallback tone={event.tone} group={event.group} iconClassName="h-3.5 w-3.5" />
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
                    {event.stageCount} ops{event.bossCount > 0 ? ` · ${event.bossCount} boss` : ""}
                </span>
                <span
                    className="pointer-events-none relative z-1 hidden h-5.25 flex-none items-center whitespace-nowrap rounded-full border px-2.5 font-mono font-semibold text-[9px] uppercase tracking-[0.09em] sm:inline-flex"
                    style={{ borderColor: `color-mix(in oklch, ${event.tone} 45%, transparent)`, background: `color-mix(in oklch, ${event.tone} 11%, transparent)`, color: event.tone }}
                >
                    {event.statusLabel}
                </span>
                <ChevronDown className="pointer-events-none relative z-1 h-3.75 w-3.75 flex-none text-muted-foreground transition-transform duration-200" style={{ transform: open ? "rotate(180deg)" : "rotate(0deg)" }} aria-hidden="true" />
            </div>

            <CollapsiblePanel open={open}>
                <div className="border-border border-t bg-background/40 px-3 pt-2.5 pb-4 sm:px-4 sm:pb-4.5">
                    <div className="mt-2.5 grid grid-cols-[repeat(auto-fill,minmax(126px,1fr))] gap-2.5 sm:grid-cols-[repeat(auto-fill,minmax(154px,1fr))] sm:gap-3">
                        {event.stages.map((st) => (
                            <StageCard key={st.stageId} card={st} tone={event.tone} group={event.group} />
                        ))}
                    </div>
                </div>
            </CollapsiblePanel>
        </div>
    );
}

function StageCardInner({ card, tone, group }: { card: IStageCardVM; tone: string; group: StageGroupKey }) {
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
                        className="absolute bottom-1.75 left-1.75 z-2 inline-flex h-4.5 max-w-[calc(100%-14px)] items-center overflow-hidden text-ellipsis whitespace-nowrap rounded-[5px] border px-1.75 font-bold font-mono text-[10px] leading-none"
                        style={{ borderColor: `color-mix(in oklch, ${tone} 50%, transparent)`, background: "rgba(0,0,0,0.6)", color: tone }}
                    >
                        {card.badge}
                    </span>
                )}
                {card.boss && (
                    <span
                        className="absolute top-1.75 right-1.75 z-2 inline-flex h-4.25 items-center gap-1 rounded-full px-1.75 font-bold font-mono text-[8.5px] text-white uppercase tracking-[0.08em]"
                        style={{ background: "color-mix(in oklch, var(--enemy-boss) 88%, black)", boxShadow: "0 0 10px color-mix(in oklch, var(--enemy-boss) 60%, transparent)" }}
                    >
                        <Skull className="h-2.5 w-2.5" /> Boss
                    </span>
                )}
            </div>
            <div className="flex flex-col gap-1.5 px-2.5 pt-2 pb-2.5">
                <span className="line-clamp-2 min-h-8 font-sans font-semibold text-[12.5px] text-foreground leading-tight">{card.title}</span>
                <div className="flex items-center justify-between gap-1.5">
                    <span className="font-mono text-[9.5px] text-muted-foreground">{card.apCost > 0 ? `${card.apCost} ◆ Sanity` : "Free entry"}</span>
                    {card.hasChallenge && <span className="font-mono text-[9.5px] text-muted-foreground/80">CM</span>}
                </div>
            </div>
        </>
    );
}

function StageCard({ card, tone, group }: { card: IStageCardVM; tone: string; group: StageGroupKey }) {
    const cardStyle = { ["--rowtone" as string]: tone };

    if (!card.canView) {
        return (
            <div className="flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left" style={{ ...cardStyle, cursor: "default" }}>
                <StageCardInner card={card} tone={tone} group={group} />
            </div>
        );
    }
    return (
        <Link to="/stages/$stageId" params={{ stageId: card.stageId }} className="msv-stage flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left transition-[transform,border-color,box-shadow] duration-150" style={cardStyle}>
            <StageCardInner card={card} tone={tone} group={group} />
        </Link>
    );
}

function FeaturedHero({ event, onBrowse, onDetails }: { event: IEventVM; onBrowse: () => void; onDetails: () => void }) {
    const cover = coverImage(event);
    return (
        <div className="relative mb-7 h-66 overflow-hidden rounded-2xl border border-border bg-card sm:mb-8 sm:h-60">
            <PreviewFallback tone={event.tone} group={event.group} iconClassName="h-8 w-8" />
            {cover && <StagePreview src={cover} />}
            {/* Legibility + brand wash: side sweep for wide screens, bottom anchor so text survives bright art at any width. */}
            <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(90deg, rgba(7,9,12,0.86) 0%, rgba(7,9,12,0.55) 42%, rgba(7,9,12,0.15) 100%)" }} />
            <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.85) 0%, rgba(7,9,12,0.35) 45%, rgba(7,9,12,0) 70%)" }} />
            <div className="pointer-events-none absolute inset-0" style={{ background: "radial-gradient(90% 130% at 90% 0%, color-mix(in oklch, var(--primary) 26%, transparent), transparent 55%)" }} />

            <div className="absolute inset-0 flex flex-col justify-between p-4.5 sm:p-7">
                <div className="flex items-center gap-2.5">
                    <span
                        className="inline-flex h-5.5 items-center rounded-full border px-2.5 font-mono font-semibold text-[9.5px] uppercase tracking-[0.09em]"
                        style={{ borderColor: `color-mix(in oklch, ${event.tone} 55%, transparent)`, background: `color-mix(in oklch, ${event.tone} 16%, rgba(0,0,0,0.35))`, color: event.tone }}
                    >
                        {event.statusLabel}
                    </span>
                    {event.kicker && <span className="font-mono text-[10px] text-white/55 uppercase tracking-[0.12em]">{event.kicker}</span>}
                </div>
                <div>
                    <div className="text-balance font-bold font-sans text-[25px] text-white leading-[1.02] tracking-[-0.03em] sm:max-w-[80%] sm:text-[34px]" style={{ textShadow: "0 2px 24px rgba(0,0,0,0.55)" }}>
                        {event.title}
                    </div>
                    <div className="mt-3 flex flex-wrap items-center gap-x-3.5 gap-y-2 sm:mt-3.5">
                        {event.codeRange && <span className="font-mono text-[11px] text-white/75">{event.codeRange}</span>}
                        {event.codeRange && <span className="h-3 w-px bg-white/25" />}
                        <span className="font-mono text-[11px] text-white/75">{event.stageCount} operations</span>
                        {event.bossCount > 0 && (
                            <>
                                <span className="h-3 w-px bg-white/25" />
                                <span className="font-mono text-[11px]" style={{ color: "var(--enemy-boss)" }}>
                                    {event.bossCount} {event.bossCount === 1 ? "boss" : "bosses"}
                                </span>
                            </>
                        )}
                        <button type="button" onClick={onBrowse} className="inline-flex h-10 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-lg bg-primary px-4 font-sans font-semibold text-[12.5px] text-primary-foreground shadow-sm transition-transform hover:-translate-y-px sm:ml-1 sm:h-8.5">
                            Browse stages
                            <ChevronRight className="h-3.25 w-3.25" aria-hidden="true" />
                        </button>
                        <button
                            type="button"
                            onClick={onDetails}
                            aria-haspopup="dialog"
                            className="inline-flex h-10 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-lg border border-white/25 bg-white/10 px-3.5 font-sans font-semibold text-[12.5px] text-white backdrop-blur-sm transition-colors hover:bg-white/18 sm:h-8.5"
                        >
                            <Info className="h-3.25 w-3.25" aria-hidden="true" />
                            Details
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
