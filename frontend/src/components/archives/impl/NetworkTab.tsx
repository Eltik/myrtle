import { useQuery } from "@tanstack/react-query";
import { ChevronRightIcon, LocateFixedIcon, MaximizeIcon, MinimizeIcon, MinusIcon, PlusIcon, UsersIcon } from "lucide-react";
import type * as React from "react";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { SearchField } from "#/components/SearchField";
import { Button } from "#/components/ui/button";
import { Sheet, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { archiveNetworkQueryOptions } from "#/lib/api/archives";
import { useFormatters, useGamedataServer } from "#/lib/i18n";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn, getAvatarById } from "#/lib/utils";
import type { ArchiveGroup } from "#/types/generated/ArchiveGroup";
import type { ArchiveNetwork } from "#/types/generated/ArchiveNetwork";
import type { ArchivePerson } from "#/types/generated/ArchivePerson";
import { bounds, CELL_R, centroid, directionVector, hexColor, hexPoints, type IPoint, outlineSegments, TERRITORY_R, toSvg } from "./geometry";
import { declutter, type ILabelBox, type IView, placeLabel, project } from "./labels";
import { type IInsets, type IPanZoomHandle, PanZoom } from "./PanZoom";
import { type ArchivesT, FactionLogo, FactionLogoSvg, PersonChip, peopleById, rarityColor, useArchivesT } from "./shared";

/** Space kept between the overlaid controls and the fitted content, in screen pixels. */
const CONTROL_GAP = 8;
const SEARCH_LIMIT = 8;
/** Chart hexes are drawn a little under their circumradius so the bridges between them show. */
const CELL_INSET = 0.86;
/** Least space between two territory labels, in screen pixels; a closer, smaller label is hidden. */
const LABEL_GAP = 6;
/** Labels stay this far inside the map's edges, in screen pixels. */
const STAGE_MARGIN = 6;
/** The game's `card_scale` at and above which a faction gets the larger label (Rhodes Island, the nations). */
const BIG_CARD = 0.8;

interface INetworkTabProps {
    chart: string | undefined;
    onChart: (chart: string | undefined) => void;
    onOpenFile: (id: string) => void;
}

export function NetworkTab({ chart, onChart, onOpenFile }: INetworkTabProps): React.ReactElement {
    const t = useArchivesT();
    const server = useGamedataServer();
    const { data, isLoading, isError } = useQuery(archiveNetworkQueryOptions(server));
    if (isError) return <p className="text-muted-foreground text-sm">{t("page.error")}</p>;
    if (isLoading || !data) return <div className="h-[min(70vh,560px)] animate-pulse rounded-[14px] border border-border bg-muted/40" />;
    if (!data.imported) return <p className="rounded-[14px] border border-border bg-card p-5 text-muted-foreground text-sm">{t("network.notImported")}</p>;
    return <Network data={data} chart={chart} onChart={onChart} onOpenFile={onOpenFile} server={server} t={t} />;
}

function Network({ data, chart, onChart, onOpenFile, server, t }: INetworkTabProps & { data: ArchiveNetwork; server: string; t: ArchivesT }): React.ReactElement {
    const f = useFormatters();
    const wrapRef = useRef<HTMLDivElement>(null);
    const panRef = useRef<IPanZoomHandle>(null);
    const [zoom, setZoom] = useState(1);
    const [fullscreen, setFullscreen] = useState(false);
    const [rosters, setRosters] = useState(false);
    const [query, setQuery] = useState("");
    // A search pick: its chart opens fitted, with the person ringed.
    const [picked, setPicked] = useState<string | null>(null);
    const toolbarRef = useRef<HTMLDivElement>(null);
    const hintRef = useRef<HTMLParagraphElement>(null);
    const [insets, setInsets] = useState<IInsets>({ top: 0, right: 0, bottom: 0, left: 0 });
    const [hover, setHover] = useState<string | null>(null);
    const people = useMemo(() => peopleById(data.people), [data.people]);
    const group = chart ? data.groups.find((g) => g.id === chart) : undefined;

    const content = useMemo(() => {
        if (group) {
            const pts = [...group.cells.map((c) => toSvg(c)), ...group.blocks.map((b) => toSvg(b.pos)), ...group.fillers.map((x) => toSvg(x.pos))];
            return bounds(pts, CELL_R * 1.6);
        }
        return bounds(
            data.forces.flatMap((fo) => [...fo.hexes.map((h) => toSvg(h)), toSvg(fo.card)]),
            TERRITORY_R * 2,
        );
    }, [group, data.forces]);

    useEffect(() => {
        const onChange = () => setFullscreen(document.fullscreenElement === wrapRef.current);
        document.addEventListener("fullscreenchange", onChange);
        return () => document.removeEventListener("fullscreenchange", onChange);
    }, []);

    // The toolbar and the hint line sit over the map; the fit keeps the
    // content clear of both, re-measured when they wrap or change language.
    useLayoutEffect(() => {
        const wrap = wrapRef.current;
        const bar = toolbarRef.current;
        const hint = hintRef.current;
        if (!wrap || !bar || !hint) return;
        const measure = () => {
            const top = Math.ceil(bar.offsetTop + bar.offsetHeight + CONTROL_GAP);
            const bottom = Math.ceil(wrap.clientHeight - hint.offsetTop + CONTROL_GAP / 2);
            setInsets((prev) => (prev.top === top && prev.bottom === bottom ? prev : { top, right: 0, bottom, left: 0 }));
        };
        measure();
        const ro = new ResizeObserver(measure);
        ro.observe(wrap);
        ro.observe(bar);
        ro.observe(hint);
        return () => ro.disconnect();
    }, []);

    const onZoom = useCallback((z: number) => setZoom(z), []);
    // Moving between charts by hand drops the search pick's ring.
    const goChart = useCallback(
        (id: string | undefined) => {
            setPicked(null);
            onChart(id);
        },
        [onChart],
    );
    // The file drawer and the rosters render outside the map, so leave full screen first.
    const openFile = useCallback(
        (id: string) => {
            if (document.fullscreenElement) void document.exitFullscreen();
            onOpenFile(id);
        },
        [onOpenFile],
    );

    const results = useMemo(() => {
        if (!query.trim()) return [];
        const placed = data.people.filter((p) => data.groups.some((g) => g.cells.some((c) => c.charId === p.id)));
        return searchAndRank(query, placed, (p) => ({ name: p.name, extra: p.number ?? undefined }), SEARCH_LIMIT).map((r) => r.item);
    }, [query, data]);

    const pickPerson = (p: ArchivePerson) => {
        // The chart where the person sits in a home block, else the first that places them.
        const home = data.groups.find((g) => g.cells.some((c) => c.charId === p.id && g.blocks.some((b) => b.home && b.forceId === c.forceId && b.members.includes(p.id))));
        const target = home ?? data.groups.find((g) => g.cells.some((c) => c.charId === p.id));
        if (!target) return;
        setQuery("");
        setPicked(p.id);
        onChart(target.id);
    };

    const toggleFullscreen = () => {
        if (document.fullscreenElement) void document.exitFullscreen();
        else void wrapRef.current?.requestFullscreen?.();
    };

    return (
        <div ref={wrapRef} className={cn("relative overflow-hidden rounded-[14px] border border-border bg-card", fullscreen ? "h-screen" : "h-[min(72vh,600px)] min-h-90")}>
            <div aria-hidden className="pointer-events-none absolute inset-0 opacity-50" style={{ backgroundImage: "radial-gradient(circle at 1px 1px, color-mix(in oklab, var(--muted-foreground) 25%, transparent) 1px, transparent 0)", backgroundSize: "22px 22px" }} />
            <PanZoom
                content={content}
                insets={insets}
                onZoom={onZoom}
                handle={panRef}
                className="absolute inset-0 size-full select-none"
                label={group ? group.name : t("network.territories")}
                minZoom={0.4}
                maxZoom={group ? 6 : 10}
                overlay={group ? undefined : (view) => <TerritoryLabels data={data} view={view} hover={hover} onHover={setHover} onPick={goChart} t={t} />}
            >
                {group ? <ChartLayer group={group} people={people} server={server} picked={picked} onOpenFile={openFile} /> : <TerritoriesLayer data={data} t={t} hover={hover} onHover={setHover} onPick={goChart} />}
            </PanZoom>

            <div ref={toolbarRef} className="pointer-events-none absolute inset-x-3 top-3 flex flex-wrap items-start justify-between gap-2">
                <div className="pointer-events-auto flex min-w-0 flex-col gap-2">
                    <nav aria-label={t("network.breadcrumb")} className="flex items-center gap-1 self-start rounded-md border border-border bg-background/85 px-2 py-1 font-mono text-[10.5px] uppercase tracking-[0.14em] backdrop-blur">
                        <button type="button" className={cn("relative cursor-pointer pointer-coarse:after:absolute pointer-coarse:after:-inset-x-2 pointer-coarse:after:-inset-y-3.5", group ? "text-muted-foreground hover:text-foreground" : "text-primary")} onClick={() => goChart(undefined)}>
                            {t("network.territories")}
                        </button>
                        {group ? (
                            <>
                                <ChevronRightIcon aria-hidden className="size-3 text-muted-foreground" />
                                <span className="text-primary">{group.name}</span>
                            </>
                        ) : null}
                    </nav>
                    <div className="relative w-64 max-w-[70vw]">
                        <SearchField value={query} onChange={setQuery} placeholder={t("network.search")} label={t("network.searchLabel")} className="bg-background/85 backdrop-blur" />
                        {query.trim() ? (
                            <ul className="absolute inset-x-0 top-full z-10 m-0 mt-1 max-h-72 list-none overflow-y-auto rounded-md border border-border bg-popover p-1 shadow-md">
                                {results.length === 0 ? (
                                    <li className="px-2 py-1.5 text-[12px] text-muted-foreground">{t("network.noMatch")}</li>
                                ) : (
                                    results.map((p) => (
                                        <li key={p.id}>
                                            <button type="button" className="flex w-full cursor-pointer flex-col rounded-sm px-2 py-1 text-left hover:bg-accent" onClick={() => pickPerson(p)}>
                                                <span className="text-[13px] text-foreground">{p.name}</span>
                                                <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.1em]">{[data.forces.find((fo) => fo.id === p.factionId)?.name ?? p.factionId, p.number].filter(Boolean).join(" · ")}</span>
                                            </button>
                                        </li>
                                    ))
                                )}
                            </ul>
                        ) : null}
                    </div>
                </div>
                <div className="pointer-events-auto flex items-center gap-1.5">
                    {group ? (
                        <Button
                            size="sm"
                            variant="outline"
                            className="bg-background/85 backdrop-blur"
                            onClick={() => {
                                if (document.fullscreenElement) void document.exitFullscreen();
                                setRosters(true);
                            }}
                            title={t("network.rostersHint")}
                        >
                            <UsersIcon aria-hidden className="size-3.5" />
                            {t("network.rosters")}
                        </Button>
                    ) : null}
                    <div className="flex items-center rounded-md border border-border bg-background/85 backdrop-blur">
                        <Button size="icon-xs" variant="ghost" aria-label={t("network.zoomOut")} title={t("network.zoomOut")} onClick={() => panRef.current?.zoomBy(1.25)}>
                            <MinusIcon aria-hidden />
                        </Button>
                        <span className="min-w-12 text-center font-mono text-[11px] text-foreground tabular-nums">{f.percent(zoom)}</span>
                        <Button size="icon-xs" variant="ghost" aria-label={t("network.zoomIn")} title={t("network.zoomIn")} onClick={() => panRef.current?.zoomBy(0.8)}>
                            <PlusIcon aria-hidden />
                        </Button>
                    </div>
                    <Button size="icon-sm" variant="outline" className="bg-background/85 backdrop-blur" aria-label={t("network.fit")} title={t("network.fit")} onClick={() => panRef.current?.fit()}>
                        <LocateFixedIcon aria-hidden />
                    </Button>
                    <Button size="icon-sm" variant="outline" className="bg-background/85 backdrop-blur" aria-label={fullscreen ? t("network.exitFullscreen") : t("network.fullscreen")} title={fullscreen ? t("network.exitFullscreen") : t("network.fullscreen")} onClick={toggleFullscreen}>
                        {fullscreen ? <MinimizeIcon aria-hidden /> : <MaximizeIcon aria-hidden />}
                    </Button>
                </div>
            </div>

            <p ref={hintRef} className="pointer-events-none absolute bottom-2 left-3 m-0 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.14em]">
                {group ? (group.cells.length === 0 ? t("network.emptyChart") : t("network.hintChart")) : t("network.hintMap")}
            </p>

            {group ? <RostersSheet open={rosters} onOpenChange={setRosters} group={group} people={people} server={server} onOpenFile={onOpenFile} t={t} /> : null}
        </div>
    );
}

function TerritoriesLayer({ data, t, hover, onHover, onPick }: { data: ArchiveNetwork; t: ArchivesT; hover: string | null; onHover: (id: string | null) => void; onPick: (groupId: string) => void }): React.ReactElement {
    const hexAt = useMemo(() => new Map(data.forces.flatMap((fo) => fo.hexes.map((h) => [h.index, toSvg(h)] as const))), [data.forces]);
    const centres = useMemo(() => new Map(data.forces.map((fo) => [fo.id, centroid(fo.hexes.map((h) => toSvg(h)))])), [data.forces]);
    return (
        <g>
            {data.forceLinks.map((l) => {
                const a = centres.get(l.a);
                const b = centres.get(l.b);
                return a && b ? <line key={`${l.a}-${l.b}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke="var(--muted-foreground)" strokeOpacity={0.55} strokeWidth={3} strokeDasharray="0.1 9" strokeLinecap="round" /> : null;
            })}
            {data.hexLinks.map((l) => {
                const a = hexAt.get(l.from);
                const b = hexAt.get(l.to);
                return a && b ? <line key={`${l.from}-${l.to}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke="var(--muted-foreground)" strokeOpacity={0.45} strokeWidth={2} /> : null;
            })}
            {data.forces.map((fo) => {
                const color = hexColor(fo.color);
                const centers = fo.hexes.map((h) => toSvg(h));
                const logo = toSvg(fo.logo);
                const label = t("network.forceLabel", { name: fo.name, people: fo.memberCount, hexes: fo.hexes.length });
                const size = TERRITORY_R * 1.3 * (fo.logoScale || 1);
                const lit = hover === fo.id;
                return (
                    // biome-ignore lint/a11y/useSemanticElements: an SVG group cannot be a <button>; it takes the role and key handling instead
                    <g
                        key={fo.id}
                        role="button"
                        tabIndex={0}
                        aria-label={label}
                        className="cursor-pointer outline-none [&:focus-visible_line]:stroke-[4]"
                        onClick={() => onPick(fo.groupId)}
                        onPointerEnter={() => onHover(fo.id)}
                        onPointerLeave={() => onHover(null)}
                        onFocus={() => onHover(fo.id)}
                        onBlur={() => onHover(null)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") onPick(fo.groupId);
                        }}
                    >
                        <title>{label}</title>
                        {centers.map((c) => (
                            <polygon key={`${c.x},${c.y}`} points={hexPoints(c, TERRITORY_R)} fill={color} fillOpacity={lit ? 0.42 : 0.2} stroke={color} strokeOpacity={lit ? 0.4 : 0.22} strokeWidth={1} className="transition-[fill-opacity]" />
                        ))}
                        {outlineSegments(centers, TERRITORY_R).map(([a, b]) => (
                            <line key={`${a.x},${a.y},${b.x},${b.y}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke={color} strokeWidth={lit ? 3.5 : 2} strokeLinecap="round" />
                        ))}
                        <FactionLogoSvg id={fo.id} x={logo.x - size / 2} y={logo.y - size / 2} size={size} opacity={lit ? 0.4 : 0.22} className="text-foreground" />
                    </g>
                );
            })}
        </g>
    );
}

/**
 * Territory name plates, drawn over the map at one screen size whatever the
 * zoom. Each plate is measured once it has rendered (fonts loaded), so a long
 * CJK or Hangul name takes the width it needs; plates that would overlap a
 * faction with more people are hidden, and the hovered or focused cluster's
 * plate always shows.
 */
function TerritoryLabels({ data, view, hover, onHover, onPick, t }: { data: ArchiveNetwork; view: IView; hover: string | null; onHover: (id: string | null) => void; onPick: (groupId: string) => void; t: ArchivesT }): React.ReactElement {
    const f = useFormatters();
    const refs = useRef(new Map<string, HTMLButtonElement>());
    const [sizes, setSizes] = useState<Map<string, { w: number; h: number }>>(new Map());

    // Plates keep their size at every zoom, so they are measured when their
    // text changes (another server, another site language) and once the
    // fonts have loaded, not per frame.
    useLayoutEffect(() => {
        void t;
        const measure = () =>
            setSizes(
                new Map(
                    data.forces.flatMap((fo) => {
                        const el = refs.current.get(fo.id);
                        return el ? [[fo.id, { w: el.offsetWidth, h: el.offsetHeight }] as const] : [];
                    }),
                ),
            );
        measure();
        let alive = true;
        void document.fonts?.ready.then(() => {
            if (alive) measure();
        });
        return () => {
            alive = false;
        };
    }, [data.forces, t]);

    const people = useMemo(() => new Map(data.forces.map((fo) => [fo.id, fo.memberCount * 1000 + fo.hexes.length])), [data.forces]);
    const boxes = data.forces.flatMap((fo): ILabelBox[] => {
        const size = sizes.get(fo.id);
        const box = size ? placeLabel(fo.id, project(toSvg(fo.card), view), size.w, size.h, view, STAGE_MARGIN) : null;
        return box ? [box] : [];
    });
    const shown = declutter(boxes, (id) => people.get(id) ?? 0, LABEL_GAP, hover);
    const at = new Map(boxes.map((b) => [b.id, b]));

    return (
        <>
            {data.forces.map((fo) => {
                const box = at.get(fo.id);
                const visible = box !== undefined && shown.has(fo.id);
                const color = hexColor(fo.cardColor, hexColor(fo.color));
                const lit = hover === fo.id;
                return (
                    <button
                        key={fo.id}
                        ref={(el) => {
                            if (el) refs.current.set(fo.id, el);
                            else refs.current.delete(fo.id);
                        }}
                        type="button"
                        tabIndex={-1}
                        aria-hidden
                        onClick={() => onPick(fo.groupId)}
                        onPointerEnter={() => onHover(fo.id)}
                        onPointerLeave={() => onHover(null)}
                        className={cn(
                            "absolute top-0 left-0 flex w-max cursor-pointer flex-col whitespace-nowrap rounded-[3px] border-l-2 text-left shadow-sm backdrop-blur-[2px] transition-[background-color,box-shadow]",
                            visible ? "pointer-events-auto" : "invisible",
                            lit ? "z-10 bg-background ring-1 ring-border" : "bg-background/85",
                        )}
                        style={{ borderLeftColor: color, transform: box ? `translate(${Math.round(box.x)}px, ${Math.round(box.y)}px)` : undefined }}
                    >
                        <span className={cn("px-2 pt-1 pb-0.5 font-semibold text-foreground leading-tight", fo.cardScale >= BIG_CARD ? "text-[13px]" : "text-[11.5px]")}>{fo.name}</span>
                        <span className="border-border/60 border-t px-2 pt-0.5 pb-1 font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.08em]">
                            {fo.code} · <span style={{ color }}>{t("network.forceCounts", { people: f.number(fo.memberCount), hexes: fo.hexes.length })}</span>
                        </span>
                    </button>
                );
            })}
        </>
    );
}

function Bridge({ from, direction, length, color }: { from: IPoint; direction: number; length: number; color: string }): React.ReactElement {
    const v = directionVector(direction);
    return <line x1={from.x} y1={from.y} x2={from.x + v.x * length} y2={from.y + v.y * length} stroke={color} strokeOpacity={0.55} strokeWidth={CELL_R * 0.2} strokeLinecap="round" />;
}

function ChartLayer({ group, people, server, picked, onOpenFile }: { group: ArchiveGroup; people: Map<string, ArchivePerson>; server: string; picked: string | null; onOpenFile: (id: string) => void }): React.ReactElement {
    const colorOf = useMemo(() => new Map(group.blocks.map((b) => [b.forceId, hexColor(b.color)])), [group.blocks]);
    const cellAt = useMemo(() => new Map(group.cells.map((c) => [c.charId, toSvg(c)])), [group.cells]);
    // Half the distance between neighbours: each side of a person-to-person
    // link draws its half. A link into a block or from a filler hex is drawn
    // whole, because the other end lists no link back.
    const bridge = CELL_R * Math.sqrt(3) * 0.5;
    const clipId = `arc-hex-clip-${group.id}`;
    return (
        <g>
            <defs>
                <clipPath id={clipId} clipPathUnits="objectBoundingBox">
                    <polygon points="0.25,0.067 0.75,0.067 1,0.5 0.75,0.933 0.25,0.933 0,0.5" />
                </clipPath>
            </defs>
            {group.fillers.map((x) => x.edges.map((e) => <Bridge key={`${x.id}-${e.direction}-${e.charId ?? e.forceId}`} from={toSvg(x.pos)} direction={e.direction} length={bridge * 2} color={hexColor(x.color, colorOf.get(x.forceId))} />))}
            {group.cells.map((c) => c.edges.map((e) => <Bridge key={`${c.charId}-${e.direction}-${e.charId ?? e.forceId}`} from={toSvg(c)} direction={e.direction} length={e.forceId ? bridge * 2 : bridge * c.scale} color={colorOf.get(c.forceId ?? "") ?? "#8a8f98"} />))}
            {group.lines.map((l) => {
                const a = cellAt.get(l.a);
                const b = cellAt.get(l.b);
                return a && b ? <line key={`${l.a}-${l.b}`} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke="var(--muted-foreground)" strokeOpacity={0.5} strokeWidth={3} /> : null;
            })}
            {group.fillers.map((x) => {
                const p = toSvg(x.pos);
                return <polygon key={x.id} points={hexPoints(p, CELL_R * CELL_INSET)} fill={hexColor(x.color, colorOf.get(x.forceId))} fillOpacity={0.35} />;
            })}
            {group.blocks.map((b) => {
                const p = toSvg(b.pos);
                const color = hexColor(b.color);
                const size = CELL_R * 0.9;
                return (
                    <g key={b.forceId}>
                        <polygon points={hexPoints(p, CELL_R * CELL_INSET)} fill={color} fillOpacity={0.18} stroke={color} strokeWidth={3} strokeDasharray={b.home ? undefined : "10 8"} />
                        <FactionLogoSvg id={b.forceId} x={p.x - size / 2} y={p.y - size / 2 - 12} size={size} opacity={0.5} className="text-foreground" />
                        <text x={p.x} y={p.y + CELL_R * 0.55} textAnchor="middle" className="fill-foreground font-semibold text-[17px]">
                            {b.name}
                        </text>
                        <text x={p.x} y={p.y + CELL_R * 0.55 + 18} textAnchor="middle" className="fill-muted-foreground font-mono text-[13px]">
                            {b.members.length}
                        </text>
                    </g>
                );
            })}
            {group.cells.map((c) => {
                const p = toSvg(c);
                const r = CELL_R * CELL_INSET * c.scale;
                const person = people.get(c.charId);
                const color = colorOf.get(c.forceId ?? "") ?? "#8a8f98";
                const img = r * 1.25;
                const open = person && !person.npc ? () => onOpenFile(c.charId) : undefined;
                const name = person?.name ?? c.charId;
                const body = (
                    <>
                        <title>{name}</title>
                        {c.charId === picked ? <polygon points={hexPoints(p, r * 1.22)} fill="none" stroke="var(--primary)" strokeWidth={6} className="animate-pulse" /> : null}
                        <polygon points={hexPoints(p, r)} fill="var(--card)" stroke={color} strokeWidth={c.outline ? 4 : 2} />
                        <polygon points={hexPoints(p, r)} fill={color} fillOpacity={0.16} />
                        {person && !person.npc ? <image href={getAvatarById(c.charId, server)} x={p.x - img / 2} y={p.y - img / 2 - r * 0.12} width={img} height={img} clipPath={`url(#${clipId})`} preserveAspectRatio="xMidYMid slice" /> : null}
                        {person?.number ? (
                            <text x={p.x - r * 0.5} y={p.y - r * 0.62} className="fill-muted-foreground font-mono text-[12px]">
                                {person.number}
                            </text>
                        ) : null}
                        <rect x={p.x - r * 0.75} y={p.y + r * 0.42} width={r * 1.5} height={22} rx={3} fill="var(--background)" fillOpacity={0.85} />
                        <rect x={p.x - r * 0.75} y={p.y + r * 0.42} width={3} height={22} fill={rarityColor(person?.rarity)} />
                        <text x={p.x} y={p.y + r * 0.42 + 15.5} textAnchor="middle" className="fill-foreground font-medium text-[13px]">
                            {name.length > 16 ? `${name.slice(0, 15)}…` : name}
                        </text>
                    </>
                );
                if (!open) return <g key={c.charId}>{body}</g>;
                return (
                    // biome-ignore lint/a11y/useSemanticElements: an SVG group cannot be a <button>; it takes the role and key handling instead
                    <g
                        key={c.charId}
                        role="button"
                        tabIndex={0}
                        aria-label={name}
                        className="cursor-pointer outline-none [&:focus-visible>polygon]:stroke-[6] [&:hover>polygon]:stroke-[6]"
                        onClick={open}
                        onKeyDown={(e) => {
                            if (e.key === "Enter" || e.key === " ") open();
                        }}
                    >
                        {body}
                    </g>
                );
            })}
        </g>
    );
}

function RostersSheet({ open, onOpenChange, group, people, server, onOpenFile, t }: { open: boolean; onOpenChange: (open: boolean) => void; group: ArchiveGroup; people: Map<string, ArchivePerson>; server: string; onOpenFile: (id: string) => void; t: ArchivesT }): React.ReactElement {
    const blocks = [...group.blocks].sort((a, b) => Number(b.home) - Number(a.home));
    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetPopup side="right" className="w-[min(420px,100vw)] sm:max-w-105">
                <SheetHeader>
                    <SheetTitle>{t("network.rosters")}</SheetTitle>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-5 pb-8">
                    {blocks.map((b) => (
                        <section key={b.forceId} className="flex flex-col gap-2">
                            <header className="flex items-center gap-2 border-border border-b pb-1.5">
                                <FactionLogo id={b.forceId} className="size-5" />
                                <h3 className="m-0 flex-1 font-semibold text-[13px] text-foreground uppercase tracking-[0.08em]">{b.name}</h3>
                                {b.home ? null : <span className="rounded-sm border border-border px-1.5 font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.12em]">{t("network.visiting")}</span>}
                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("network.rosterCount", { count: b.members.length })}</span>
                            </header>
                            <div className="flex flex-wrap gap-1.5">
                                {b.members.map((id) => (
                                    <PersonChip key={id} person={people.get(id)} onOpen={onOpenFile} server={server} />
                                ))}
                            </div>
                        </section>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    );
}
