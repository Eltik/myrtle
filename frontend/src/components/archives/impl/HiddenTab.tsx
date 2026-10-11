import { useQuery } from "@tanstack/react-query";
import { CornerDownRightIcon, SearchIcon } from "lucide-react";
import type * as React from "react";
import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { archiveHiddenFactionsQueryOptions } from "#/lib/api/archives";
import { useFormatters, useGamedataServer } from "#/lib/i18n";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn } from "#/lib/utils";
import type { ArchiveHiddenFactions } from "#/types/generated/ArchiveHiddenFactions";
import type { ArchivePerson } from "#/types/generated/ArchivePerson";
import type { ArchiveSecret } from "#/types/generated/ArchiveSecret";
import { FLOW_GAP, type IFlowNode, layoutFlow } from "./flow";
import { FactionLogo, Panel, PersonChip, peopleById, useArchivesT } from "./shared";

type Pick = { kind: "shown"; id: string } | { kind: "hidden"; id: string } | { kind: "band"; shown: string; hidden: string };

function samePick(a: Pick | null, b: Pick): boolean {
    if (!a || a.kind !== b.kind) return false;
    if (a.kind === "band" && b.kind === "band") return a.shown === b.shown && a.hidden === b.hidden;
    return a.kind !== "band" && b.kind !== "band" && a.id === b.id;
}

/** Whether a band (one shown/hidden pairing) is part of a pick or hover. */
function covers(p: Pick | null, shown: string, hidden: string): boolean {
    if (!p) return false;
    if (p.kind === "shown") return p.id === shown;
    if (p.kind === "hidden") return p.id === hidden;
    return p.shown === shown && p.hidden === hidden;
}

const bandKey = (shown: string, hidden: string) => `${shown}\u0000${hidden}`;

export function HiddenTab({ onOpenFile }: { onOpenFile: (id: string) => void }): React.ReactElement {
    const t = useArchivesT();
    const server = useGamedataServer();
    const { data, isLoading, isError } = useQuery(archiveHiddenFactionsQueryOptions(server));
    if (isError) return <p className="text-muted-foreground text-sm">{t("page.error")}</p>;
    if (isLoading || !data) return <p className="text-muted-foreground text-sm">{t("page.loading")}</p>;
    return <Hidden data={data} onOpenFile={onOpenFile} server={server} />;
}

function Hidden({ data, onOpenFile, server }: { data: ArchiveHiddenFactions; onOpenFile: (id: string) => void; server: string }): React.ReactElement {
    const t = useArchivesT();
    const f = useFormatters();
    const people = useMemo(() => peopleById(data.people), [data.people]);
    const [query, setQuery] = useState("");
    const deferred = useDeferredValue(query.trim());
    const [pick, setPick] = useState<Pick | null>(null);

    const names = useMemo(() => {
        const m = new Map<string, string>();
        for (const s of data.secret) {
            m.set(`shown:${s.shown.id}`, s.shown.name);
            m.set(`hidden:${s.hidden.id}`, s.hidden.name);
        }
        return m;
    }, [data.secret]);
    const shownName = useCallback((id: string) => names.get(`shown:${id}`) ?? id, [names]);
    const hiddenName = useCallback((id: string) => names.get(`hidden:${id}`) ?? id, [names]);

    const matches = useMemo(() => (deferred ? searchAndRank(deferred, data.secret, (s) => ({ name: people.get(s.charId)?.name ?? s.charId })).map((r) => r.item) : []), [deferred, data.secret, people]);
    const lit = useMemo(() => new Set(matches.map((s) => bandKey(s.shown.id, s.hidden.id))), [matches]);

    const picked: ArchiveSecret[] = useMemo(() => (pick ? data.secret.filter((s) => covers(pick, s.shown.id, s.hidden.id)) : []), [pick, data.secret]);

    const operators = new Set(data.secret.map((s) => s.charId)).size;
    const hiddenFactions = new Set(data.secret.map((s) => s.hidden.id)).size;

    return (
        <div className="flex flex-col gap-6">
            <header className="flex flex-col gap-3">
                <Kicker className="mb-0">{t("hidden.kicker")}</Kicker>
                <p className="m-0 max-w-3xl text-[13.5px] text-muted-foreground leading-normal">{t("hidden.lede")}</p>
                <div className="grid grid-cols-4 overflow-hidden rounded-[14px] border border-border bg-card max-[760px]:grid-cols-2">
                    <Stat label={t("hidden.stat.factions")} value={f.number(hiddenFactions)} />
                    <Stat label={t("hidden.stat.operators")} value={f.number(operators)} />
                    <Stat label={t("hidden.stat.links")} value={f.number(data.secret.length)} />
                    <Stat label={t("hidden.stat.nations")} value={f.number(data.nations.length)} />
                </div>
            </header>

            <section className="flex flex-col gap-3">
                <SectionHead index="01" title={t("hidden.s1.title")} blurb={t("hidden.s1.blurb")} />
                <div className="flex flex-wrap items-center gap-2 rounded-[14px] border border-border bg-card p-2">
                    <div className="relative min-w-0 flex-1 basis-56">
                        <SearchIcon aria-hidden className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
                        <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("hidden.search")} aria-label={t("hidden.search")} className="pl-8" />
                    </div>
                    {deferred ? <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("hidden.searchCount", { shown: f.number(matches.length), total: f.number(data.secret.length) })}</span> : null}
                </div>
                <Panel>
                    <FlowChart secret={data.secret} shownName={shownName} hiddenName={hiddenName} pick={pick} onPick={(p) => setPick((cur) => (p && samePick(cur, p) ? null : p))} lit={lit} />
                    <PickList pick={pick} picked={picked} matches={deferred ? matches : null} people={people} shownName={shownName} hiddenName={hiddenName} onOpenFile={onOpenFile} onClear={() => setPick(null)} server={server} />
                </Panel>
            </section>

            <section className="flex flex-col gap-3">
                <SectionHead index="02" title={t("hidden.s2.title")} blurb={t("hidden.s2.blurb")} />
                <div className="grid grid-cols-2 gap-4 max-[800px]:grid-cols-1">
                    {data.nations.map((n) => {
                        const groups = new Map<string, { name: string; ids: string[] }>();
                        for (const m of n.members) {
                            const g = groups.get(m.shown.id) ?? { name: m.shown.name, ids: [] };
                            g.ids.push(m.charId);
                            groups.set(m.shown.id, g);
                        }
                        return (
                            <Panel key={n.nation.id}>
                                <div className="flex items-center gap-2">
                                    <FactionLogo id={n.nation.id} className="size-7" />
                                    <h3 className="m-0 flex-1 font-semibold text-[15px] text-foreground">{n.nation.name}</h3>
                                    <span className="font-mono text-[12px] text-muted-foreground tabular-nums">{f.number(n.members.length)}</span>
                                </div>
                                {[...groups.entries()].map(([gid, g]) => (
                                    <div key={gid} className="flex flex-col gap-1.5">
                                        <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{t("hidden.cardShowsGroup", { name: g.name })}</span>
                                        <div className="flex flex-wrap gap-1.5">
                                            {g.ids.map((id) => (
                                                <PersonChip key={id} person={people.get(id)} onOpen={onOpenFile} server={server} />
                                            ))}
                                        </div>
                                    </div>
                                ))}
                            </Panel>
                        );
                    })}
                </div>
            </section>
        </div>
    );
}

/** Width of the middle (band) column, measured, so band ends land exactly on the bars. */
function useWidth(): [React.RefObject<HTMLDivElement | null>, number] {
    const ref = useRef<HTMLDivElement>(null);
    const [w, setW] = useState(0);
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const ro = new ResizeObserver(([entry]) => setW(entry.contentRect.width));
        ro.observe(el);
        return () => ro.disconnect();
    }, []);
    return [ref, w];
}

/** Width of a node's bar at each end of the bands. */
const BAR = 4;

function FlowChart({ secret, shownName, hiddenName, pick, onPick, lit }: { secret: readonly ArchiveSecret[]; shownName: (id: string) => string; hiddenName: (id: string) => string; pick: Pick | null; onPick: (p: Pick | null) => void; lit: ReadonlySet<string> }): React.ReactElement {
    const t = useArchivesT();
    const f = useFormatters();
    const [hover, setHover] = useState<Pick | null>(null);
    const [midRef, width] = useWidth();
    const layout = useMemo(
        () =>
            layoutFlow(
                secret.map((s) => ({ shown: s.shown.id, hidden: s.hidden.id, count: 1 })),
                (id) => shownName(id) || hiddenName(id),
            ),
        [secret, shownName, hiddenName],
    );
    const related = useMemo(() => {
        const out = new Set<string>();
        if (!pick) return out;
        for (const b of layout.bands) {
            if (!covers(pick, b.shown, b.hidden)) continue;
            out.add(`shown:${b.shown}`);
            out.add(`hidden:${b.hidden}`);
        }
        return out;
    }, [pick, layout.bands]);
    const pickedBands = pick ? layout.bands.filter((b) => covers(pick, b.shown, b.hidden)) : [];
    const path = (y1: number, y2: number) => `M${BAR} ${y1} C${width / 2} ${y1}, ${width / 2} ${y2}, ${width - BAR} ${y2}`;

    const label = (side: "shown" | "hidden", node: IFlowNode) => {
        const p: Pick = side === "shown" ? { kind: "shown", id: node.id } : { kind: "hidden", id: node.id };
        const name = side === "shown" ? shownName(node.id) : hiddenName(node.id);
        const isPicked = samePick(pick, p);
        const isRelated = related.has(`${side}:${node.id}`);
        const left = side === "shown";
        return (
            <div key={node.id} className={cn("flex items-center", left ? "justify-end" : "justify-start")} style={{ height: node.slot, marginBottom: FLOW_GAP }}>
                <button
                    type="button"
                    aria-pressed={isPicked}
                    aria-label={t("hidden.nodeLabel", { name, count: node.count })}
                    title={t("hidden.nodeLabel", { name, count: node.count })}
                    onClick={() => onPick(p)}
                    onPointerEnter={() => setHover(p)}
                    onPointerLeave={() => setHover(null)}
                    onFocus={() => setHover(p)}
                    onBlur={() => setHover(null)}
                    className={cn(
                        "inline-flex h-6 cursor-pointer items-center gap-1.5 whitespace-nowrap rounded-md px-2 text-[12px] outline-none transition-[opacity,color,background-color] focus-visible:ring-2 focus-visible:ring-ring",
                        left ? "flex-row" : "flex-row-reverse",
                        isPicked ? "bg-primary font-semibold text-primary-foreground" : "text-foreground hover:text-primary",
                        pick && !isPicked && !isRelated ? "opacity-40" : "",
                    )}
                >
                    <span className={cn("font-mono tabular-nums", isPicked ? "" : "text-muted-foreground")}>{f.number(node.count)}</span>
                    <FactionLogo id={node.id} className="size-3.5" />
                    <span className="font-medium">{name}</span>
                </button>
            </div>
        );
    };

    // Under 640 px the chart would need 520 px of width and a sideways scroll
    // whose bar sits two thousand pixels down, so a phone gets the same picks
    // as a list instead: each faction the card shows, then the factions it
    // secretly reaches. Same buttons, same picks, no bands.
    const bandsFrom = (id: string) => layout.bands.filter((b) => b.shown === id);
    const list = (
        <ul className="m-0 flex list-none flex-col p-0 sm:hidden">
            <li className="flex justify-between pb-1 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                <span>{t("hidden.cardShows")}</span>
                <span>{t("hidden.secretlyAlso")}</span>
            </li>
            {layout.left.map((n) => (
                <li key={n.id} className="flex flex-col gap-1 border-border border-t py-2">
                    <div className="[&>div]:justify-start! [&>div]:mb-0! [&>div]:h-auto! [&_button]:min-h-11">{label("shown", n)}</div>
                    <div className="flex flex-wrap gap-1.5 pl-5">
                        {bandsFrom(n.id).map((b) => {
                            const p: Pick = { kind: "band", shown: b.shown, hidden: b.hidden };
                            const isPicked = samePick(pick, p);
                            const on = lit.has(bandKey(b.shown, b.hidden));
                            const text = t("hidden.bandLabel", { count: b.count, shown: shownName(b.shown), hidden: hiddenName(b.hidden) });
                            return (
                                <button
                                    key={bandKey(b.shown, b.hidden)}
                                    type="button"
                                    aria-pressed={isPicked}
                                    aria-label={text}
                                    title={text}
                                    onClick={() => onPick(p)}
                                    className={cn(
                                        "inline-flex min-h-11 cursor-pointer items-center gap-1.5 rounded-md border px-2.5 text-[12px] outline-none transition-[opacity,color,background-color] focus-visible:ring-2 focus-visible:ring-ring",
                                        isPicked ? "border-primary bg-primary font-semibold text-primary-foreground" : on ? "border-primary text-primary" : "border-border text-foreground",
                                        pick && !isPicked && !covers(pick, b.shown, b.hidden) ? "opacity-40" : "",
                                    )}
                                >
                                    <CornerDownRightIcon aria-hidden className="size-3.5 opacity-60" />
                                    <FactionLogo id={b.hidden} className="size-3.5" />
                                    <span className="font-medium">{hiddenName(b.hidden)}</span>
                                    <span className={cn("font-mono tabular-nums", isPicked ? "" : "text-muted-foreground")}>{f.number(b.count)}</span>
                                </button>
                            );
                        })}
                    </div>
                </li>
            ))}
        </ul>
    );

    return (
        // biome-ignore lint/a11y/noStaticElementInteractions: Escape anywhere in the chart clears the pick; the controls inside are the buttons
        <div
            onKeyDown={(e) => {
                if (e.key === "Escape") onPick(null);
            }}
        >
            {list}
            <div className="overflow-x-auto max-sm:hidden">
                <div className="grid min-w-[520px] grid-cols-[max-content_minmax(120px,1fr)_max-content] gap-x-2">
                    <span className="pb-2 text-right font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{t("hidden.cardShows")}</span>
                    <span />
                    <span className="pb-2 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{t("hidden.secretlyAlso")}</span>

                    <div style={{ paddingTop: layout.left[0]?.top ?? 0 }}>{layout.left.map((n) => label("shown", n))}</div>
                    <div ref={midRef} className="min-w-0">
                        {width > 0 ? (
                            // biome-ignore lint/a11y/useSemanticElements: an SVG cannot be a <fieldset>; the group names the focusable bands inside it
                            <svg role="group" aria-label={t("hidden.chartLabel")} width={width} height={layout.height} className="block overflow-visible">
                                <g className={cn("transition-opacity", pick ? "opacity-35" : "")}>
                                    {layout.bands.map((b) => {
                                        const p: Pick = { kind: "band", shown: b.shown, hidden: b.hidden };
                                        const on = covers(hover, b.shown, b.hidden) || lit.has(bandKey(b.shown, b.hidden));
                                        const text = t("hidden.bandLabel", { count: b.count, shown: shownName(b.shown), hidden: hiddenName(b.hidden) });
                                        return (
                                            // biome-ignore lint/a11y/useSemanticElements: an SVG band cannot be a <button>; it takes the role and key handling instead
                                            <path
                                                key={bandKey(b.shown, b.hidden)}
                                                d={path(b.y1, b.y2)}
                                                fill="none"
                                                stroke={on ? "var(--primary)" : "var(--foreground)"}
                                                strokeOpacity={on ? 0.55 : 0.14}
                                                strokeWidth={b.width}
                                                role="button"
                                                tabIndex={0}
                                                aria-label={text}
                                                aria-pressed={samePick(pick, p)}
                                                className="cursor-pointer outline-none transition-[stroke-opacity]"
                                                style={{ pointerEvents: "stroke" }}
                                                onClick={() => onPick(p)}
                                                onPointerEnter={() => setHover(p)}
                                                onPointerLeave={() => setHover(null)}
                                                onFocus={() => setHover(p)}
                                                onBlur={() => setHover(null)}
                                                onKeyDown={(e) => {
                                                    if (e.key === "Enter" || e.key === " ") {
                                                        e.preventDefault();
                                                        onPick(p);
                                                    }
                                                }}
                                            >
                                                <title>{text}</title>
                                            </path>
                                        );
                                    })}
                                </g>
                                {/* The pick, drawn solid on top of the dimmed rest. */}
                                <g className="pointer-events-none">
                                    {pickedBands.map((b) => (
                                        <path key={bandKey(b.shown, b.hidden)} d={path(b.y1, b.y2)} fill="none" stroke="var(--primary)" strokeOpacity={0.92} strokeWidth={b.width} />
                                    ))}
                                </g>
                                <g className="pointer-events-none">
                                    {layout.left.map((n) => (
                                        <rect key={n.id} x={0} y={n.barTop} width={BAR} height={n.barHeight} fill={related.has(`shown:${n.id}`) ? "var(--primary)" : "var(--foreground)"} fillOpacity={related.has(`shown:${n.id}`) ? 1 : 0.6} />
                                    ))}
                                    {layout.right.map((n) => (
                                        <rect key={n.id} x={width - BAR} y={n.barTop} width={BAR} height={n.barHeight} fill={related.has(`hidden:${n.id}`) ? "var(--primary)" : "var(--foreground)"} fillOpacity={related.has(`hidden:${n.id}`) ? 1 : 0.6} />
                                    ))}
                                </g>
                            </svg>
                        ) : (
                            <div style={{ height: layout.height }} />
                        )}
                    </div>
                    <div style={{ paddingTop: layout.right[0]?.top ?? 0 }}>{layout.right.map((n) => label("hidden", n))}</div>
                </div>
                <p className="m-0 mt-1 text-[11px] text-muted-foreground">{t("hidden.chartHint")}</p>
            </div>
        </div>
    );
}

/** Groups of operators under the chart, largest first. */
function groupBy(rows: readonly ArchiveSecret[], key: (s: ArchiveSecret) => string): [string, ArchiveSecret[]][] {
    const m = new Map<string, ArchiveSecret[]>();
    for (const s of rows) {
        const k = key(s);
        const list = m.get(k) ?? [];
        list.push(s);
        m.set(k, list);
    }
    return [...m.entries()].sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]));
}

function PickList({
    pick,
    picked,
    matches,
    people,
    shownName,
    hiddenName,
    onOpenFile,
    onClear,
    server,
}: {
    pick: Pick | null;
    picked: readonly ArchiveSecret[];
    matches: readonly ArchiveSecret[] | null;
    people: Map<string, ArchivePerson>;
    shownName: (id: string) => string;
    hiddenName: (id: string) => string;
    onOpenFile: (id: string) => void;
    onClear: () => void;
    server: string;
}): React.ReactElement {
    const t = useArchivesT();
    const f = useFormatters();
    const chips = (rows: readonly ArchiveSecret[]) => (
        <div className="flex flex-wrap gap-1.5">
            {rows.map((s) => (
                <PersonChip key={`${s.charId}:${s.hidden.id}`} person={people.get(s.charId)} onOpen={onOpenFile} server={server} />
            ))}
        </div>
    );
    const sub = (title: string, id: string, rows: readonly ArchiveSecret[]) => (
        <div key={id} className="flex flex-col gap-1.5">
            <span className="inline-flex items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                <FactionLogo id={id} className="size-3.5" />
                {title}
                <span className="tabular-nums">{f.number(rows.length)}</span>
            </span>
            {chips(rows)}
        </div>
    );
    let body: React.ReactNode;
    if (pick) {
        const head = pick.kind === "shown" ? t("hidden.cardShowsGroup", { name: shownName(pick.id) }) : pick.kind === "hidden" ? hiddenName(pick.id) : t("hidden.bandHeading", { shown: shownName(pick.shown), hidden: hiddenName(pick.hidden) });
        body = (
            <>
                <div className="flex items-center gap-2">
                    {pick.kind === "band" ? null : <FactionLogo id={pick.id} className="size-5" />}
                    <h4 className="m-0 font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{head}</h4>
                    <span className="font-mono text-[12px] text-muted-foreground tabular-nums">{f.number(picked.length)}</span>
                    <Button size="xs" variant="ghost" className="ml-auto" onClick={onClear}>
                        {t("hidden.clear")}
                    </Button>
                </div>
                {pick.kind === "band"
                    ? chips(picked)
                    : pick.kind === "shown"
                      ? groupBy(picked, (s) => s.hidden.id).map(([id, rows]) => sub(t("hidden.secretlyGroup", { name: hiddenName(id) }), id, rows))
                      : groupBy(picked, (s) => s.shown.id).map(([id, rows]) => sub(t("hidden.cardShowsGroup", { name: shownName(id) }), id, rows))}
            </>
        );
    } else if (matches) {
        body =
            matches.length === 0 ? (
                <p className="m-0 text-[12px] text-muted-foreground">{t("hidden.noMatch")}</p>
            ) : (
                <>
                    <h4 className="m-0 font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{t("hidden.found", { count: f.number(matches.length) })}</h4>
                    {groupBy(matches, (s) => s.hidden.id).map(([id, rows]) => sub(t("hidden.secretlyGroup", { name: hiddenName(id) }), id, rows))}
                </>
            );
    } else {
        body = <p className="m-0 text-[12px] text-muted-foreground">{t("hidden.pick")}</p>;
    }
    return (
        // On a phone the picks come from a list 3,236 px tall, so the result
        // sits above it and sticks under the site header while the list
        // scrolls, instead of waiting at the foot of the list.
        <div aria-live="polite" className="flex min-h-14 flex-col gap-3 border-border border-t pt-3 max-sm:sticky max-sm:top-14 max-sm:z-1 max-sm:order-first max-sm:max-h-[40dvh] max-sm:overflow-y-auto max-sm:border-t-0 max-sm:border-b max-sm:bg-card max-sm:pt-0 max-sm:pb-3">
            {body}
        </div>
    );
}

function Stat({ label, value }: { label: string; value: string }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5 border-border not-last:border-r px-4 py-3.5 max-[760px]:nth-[2n]:border-r-0">
            <span className="font-bold text-[28px] text-foreground tabular-nums leading-none tracking-tight">{value}</span>
            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">{label}</span>
        </div>
    );
}

function SectionHead({ index, title, blurb }: { index: string; title: string; blurb: string }): React.ReactElement {
    return (
        <header className="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-border border-b pb-2">
            <span className="font-bold font-mono text-primary text-sm">{index}</span>
            <h2 className="m-0 font-semibold text-[20px] text-foreground tracking-tight">{title}</h2>
            <p className="m-0 text-[13px] text-muted-foreground">{blurb}</p>
        </header>
    );
}
