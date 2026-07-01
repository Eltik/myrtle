import { useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronRight, Search, Skull } from "lucide-react";
import { useMemo, useState } from "react";
import { stageIndexQueryOptions } from "#/lib/api/stages";
import type { StageGroupKey } from "#/lib/registry/stage-groups";
import { buildStageTree, filterTree, type IEventVM, type IGroupVM, type IStageCardVM } from "./impl/derive";
import { StagePreview } from "./impl/StagePreview";

function coverImage(event: IEventVM): string | null {
    return event.banner ?? event.stages.find((s) => s.preview)?.preview ?? null;
}

export function StageList() {
    const { data: entries } = useSuspenseQuery(stageIndexQueryOptions());
    const tree = useMemo(() => buildStageTree(entries), [entries]);

    const [q, setQ] = useState("");
    const [filter, setFilter] = useState<StageGroupKey | "all">("all");
    const [open, setOpen] = useState<Record<string, boolean>>(() => (tree.featured ? { [tree.featured.zoneId]: true } : {}));

    const searching = q.trim().length > 0;
    const visibleGroups = useMemo(() => filterTree(tree, q, filter), [tree, q, filter]);
    const visibleEvents = useMemo(() => visibleGroups.flatMap((g) => g.events), [visibleGroups]);
    const visibleStageCount = useMemo(() => visibleEvents.reduce((n, e) => n + e.stageCount, 0), [visibleEvents]);

    const isOpen = (zoneId: string) => searching || !!open[zoneId];
    const toggle = (zoneId: string) => setOpen((s) => ({ ...s, [zoneId]: s[zoneId]! }));
    const expandAll = () => setOpen(Object.fromEntries(visibleEvents.map((e) => [e.zoneId, true])));
    const collapseAll = () => setOpen({});

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

            {/* Masthead */}
            <div className="mt-6 mb-7 flex flex-wrap items-end justify-between gap-6 border-border border-b pb-6">
                <div>
                    <div className="mb-3 flex items-center gap-2.5">
                        <span className="h-px w-5" style={{ background: "var(--primary)" }} />
                        <span className="font-mono font-semibold text-[10.5px] text-primary uppercase tracking-[0.24em]">The Stage Record</span>
                    </div>
                    <h1 className="m-0 text-balance font-bold font-sans text-[34px] text-foreground leading-[0.98] tracking-[-0.035em]">Every operation, catalogued.</h1>
                    <p className="mt-3 max-w-140 text-pretty font-sans text-[13px] text-muted-foreground leading-[1.55]">Browse by story arc and code - each stage opens to a faux-3D board and enemy-pathing simulator. Ordered by episode and code.</p>
                </div>
                <div className="text-right">
                    <div className="font-light font-mono text-[34px] text-primary leading-none tracking-[-0.02em]">{tree.totalStages.toLocaleString()}</div>
                    <div className="mt-1.5 font-mono text-[10px] text-muted-foreground tracking-[0.12em]">
                        STAGES · {tree.totalZones} ZONES · {tree.groups.length} CATEGORIES
                    </div>
                </div>
            </div>

            {/* Featured hero */}
            {!searching && tree.featured && (filter === "all" || filter === tree.featured.group) && (
                <FeaturedHero
                    event={tree.featured}
                    onBrowse={() => {
                        const zoneId = tree.featured?.zoneId;
                        if (zoneId) setOpen((s) => ({ ...s, [zoneId]: true }));
                    }}
                />
            )}

            {/* Toolbar */}
            <div className="mb-3 flex flex-wrap items-center gap-3">
                <div className="relative min-w-60 max-w-100 flex-1">
                    <Search className="pointer-events-none absolute top-1/2 left-3.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                    <input
                        value={q}
                        onChange={(e) => setQ(e.target.value)}
                        placeholder="Search event, code or stage…"
                        aria-label="Search stages"
                        className="h-9.5 w-full rounded-[9px] border border-border bg-secondary/50 pr-3.5 pl-9 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40"
                    />
                </div>
                <span className="font-mono text-[11px] text-muted-foreground">{searching ? `${visibleStageCount} matches` : `${visibleEvents.length} zones`}</span>
                <div className="ml-auto flex gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75">
                    <button type="button" onClick={collapseAll} className="h-6.5 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] text-muted-foreground transition-colors hover:text-foreground">
                        Collapse
                    </button>
                    <button type="button" onClick={expandAll} className="h-6.5 cursor-pointer rounded-md px-2.5 font-sans font-semibold text-[11.5px] text-muted-foreground transition-colors hover:text-foreground">
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

            {/* Timeline spine */}
            <div className="relative pl-6.5">
                <div className="absolute top-2 bottom-2.5 left-0.75 w-px" style={{ background: "linear-gradient(180deg,var(--border),color-mix(in oklch,var(--border) 25%,transparent))" }} />
                <div className="flex flex-col gap-8">
                    {visibleGroups.map((g) => (
                        <GroupSection key={g.key} group={g} isOpen={isOpen} toggle={toggle} />
                    ))}
                </div>
            </div>
        </div>
    );
}

function FilterPill({ label, count, tone, active, onClick }: { label: string; count: number; tone: string; active: boolean; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="msv-pill inline-flex h-7.5 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-full border px-3 font-sans font-semibold text-[12px] transition-colors"
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

function GroupSection({ group, isOpen, toggle }: { group: IGroupVM; isOpen: (id: string) => boolean; toggle: (id: string) => void }) {
    return (
        <section style={{ scrollMarginTop: 74 }}>
            <div className="relative mb-3.5">
                <span className="absolute top-1.5 -left-5.75 h-1.75 w-1.75 rotate-45" style={{ background: group.tone, boxShadow: "0 0 0 4px var(--background)" }} />
                <div className="flex items-baseline gap-3">
                    <h2 className="m-0 font-bold font-sans text-[17px] text-foreground tracking-[-0.02em]">{group.label}</h2>
                    <span className="font-mono text-[10px] text-muted-foreground tracking-widest">
                        {group.zoneCount} zones · {group.stageCount} stages
                    </span>
                    <span className="h-px flex-1 bg-border" />
                </div>
            </div>

            <div className="overflow-hidden rounded-xl border border-border bg-card/40">
                {group.events.map((ev) => (
                    <EventRow key={ev.zoneId} event={ev} open={isOpen(ev.zoneId)} onToggle={() => toggle(ev.zoneId)} />
                ))}
            </div>
        </section>
    );
}

function EventRow({ event, open, onToggle }: { event: IEventVM; open: boolean; onToggle: () => void }) {
    const cover = coverImage(event);
    return (
        <div className="border-border border-b last:border-b-0" style={{ ["--rowtone" as string]: event.tone }}>
            <button type="button" onClick={onToggle} aria-expanded={open} className="msv-row relative flex w-full cursor-pointer items-center gap-4 px-4.5 py-3.5 text-left" style={{ background: open ? `color-mix(in oklch, ${event.tone} 7%, transparent)` : "transparent" }}>
                {open && <span className="absolute top-0 bottom-0 left-0 w-0.5 rounded-r" style={{ background: event.tone }} />}

                {/* Zone thumbnail */}
                <div className="relative h-10 w-16 flex-none overflow-hidden rounded-md border border-border bg-[#0a0d11]">
                    {cover && <StagePreview src={cover} />}
                    <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(120deg, rgba(7,9,12,0.5), rgba(7,9,12,0.05))" }} />
                </div>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2.5">
                        <span className="truncate font-[650] font-sans text-[15px] text-foreground leading-[1.15] tracking-[-0.01em]">{event.title}</span>
                        {event.kicker && <span className="whitespace-nowrap font-mono text-[10px] text-muted-foreground">{event.kicker}</span>}
                    </div>
                    {event.codeRange && <div className="mt-0.5 font-mono text-[10px] text-muted-foreground tracking-[0.04em]">{event.codeRange}</div>}
                </div>

                <span className="hidden whitespace-nowrap font-mono text-[10.5px] text-muted-foreground sm:inline">
                    {event.stageCount} ops{event.bossCount > 0 ? ` · ${event.bossCount} boss` : ""}
                </span>
                <span
                    className="inline-flex h-5.25 flex-none items-center whitespace-nowrap rounded-full border px-2.5 font-mono font-semibold text-[9px] uppercase tracking-[0.09em]"
                    style={{ borderColor: `color-mix(in oklch, ${event.tone} 45%, transparent)`, background: `color-mix(in oklch, ${event.tone} 11%, transparent)`, color: event.tone }}
                >
                    {event.statusLabel}
                </span>
                <ChevronDown className="h-3.75 w-3.75 flex-none text-muted-foreground transition-transform duration-200" style={{ transform: open ? "rotate(180deg)" : "rotate(0deg)" }} aria-hidden="true" />
            </button>

            {open && (
                <div className="border-border border-t bg-background/40 px-4 pt-2.5 pb-4.5">
                    <div className="mt-2.5 grid grid-cols-[repeat(auto-fill,minmax(154px,1fr))] gap-3">
                        {event.stages.map((st) => (
                            <StageCard key={st.stageId} card={st} tone={event.tone} />
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

function StageCardInner({ card, tone }: { card: IStageCardVM; tone: string }) {
    return (
        <>
            <div className="relative aspect-16/10 w-full flex-none bg-[#0a0d11]">
                {card.preview && <StagePreview src={card.preview} />}
                <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.9) 0%, rgba(7,9,12,0.15) 46%, rgba(7,9,12,0) 66%)" }} />
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

function StageCard({ card, tone }: { card: IStageCardVM; tone: string }) {
    const cardClass = "msv-stage flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left transition-[transform,border-color,box-shadow] duration-150";
    const cardStyle = { ["--rowtone" as string]: tone };

    // Procedural-mode nodes (IS/RA/CC/Paradox) have no viewer yet - render static.
    if (!card.canView) {
        return (
            <div className={cardClass} style={{ ...cardStyle, cursor: "default" }}>
                <StageCardInner card={card} tone={tone} />
            </div>
        );
    }
    return (
        <Link to="/stages/$stageId" params={{ stageId: card.stageId }} className={cardClass} style={cardStyle}>
            <StageCardInner card={card} tone={tone} />
        </Link>
    );
}

function FeaturedHero({ event, onBrowse }: { event: IEventVM; onBrowse: () => void }) {
    const cover = coverImage(event);
    return (
        <div className="relative mb-8 h-60 overflow-hidden rounded-2xl border border-border bg-card">
            {cover ? <StagePreview src={cover} /> : <div className="absolute inset-0" style={{ background: "linear-gradient(110deg, color-mix(in oklch, var(--primary) 20%, var(--card)) 0%, var(--card) 60%)" }} />}
            {/* Legibility + brand wash */}
            <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(90deg, rgba(7,9,12,0.86) 0%, rgba(7,9,12,0.55) 42%, rgba(7,9,12,0.15) 100%)" }} />
            <div className="pointer-events-none absolute inset-0" style={{ background: "radial-gradient(90% 130% at 90% 0%, color-mix(in oklch, var(--primary) 26%, transparent), transparent 55%)" }} />

            <div className="absolute inset-0 flex flex-col justify-between p-7">
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
                    <div className="max-w-[80%] text-balance font-bold font-sans text-[34px] text-white leading-[1.02] tracking-[-0.03em]" style={{ textShadow: "0 2px 24px rgba(0,0,0,0.55)" }}>
                        {event.title}
                    </div>
                    <div className="mt-3.5 flex flex-wrap items-center gap-x-3.5 gap-y-1.5">
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
                        <button type="button" onClick={onBrowse} className="ml-1 inline-flex h-8.5 flex-none cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-lg bg-primary px-4 font-sans font-semibold text-[12.5px] text-primary-foreground shadow-sm transition-transform hover:-translate-y-px">
                            Browse stages
                            <ChevronRight className="h-3.25 w-3.25" />
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
