import { useQuery } from "@tanstack/react-query";
import { useWindowVirtualizer } from "@tanstack/react-virtual";
import { Lock, Search } from "lucide-react";
import { memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { LevelBadge } from "#/components/enemies/list/impl/components/atoms";
import { EnemyPlaceholder } from "#/components/enemies/list/impl/components/EnemyPlaceholder";
import { LEVEL_TOKENS } from "#/components/enemies/list/impl/tokens";
import { Badge } from "#/components/ui/badge";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Skeleton } from "#/components/ui/skeleton";
import { Toggle } from "#/components/ui/toggle";
import { env } from "#/env";
import { enemiesListQueryOptions, type IEnemy, type IEnemyLevel } from "#/lib/api/enemies";
import type { IEncounteredEnemies } from "#/lib/api/user";
import { cn } from "#/lib/utils";

type StatusFilter = "all" | "encountered" | "missing";
type LevelFilter = IEnemyLevel | "ALL";

interface IEnemyRow {
    enemy: IEnemy;
    seen: boolean;
}

const STATUS_LABELS: Record<StatusFilter, string> = {
    all: "All",
    encountered: "Encountered",
    missing: "Missing",
};

const LEVEL_FILTERS: { value: LevelFilter; label: string }[] = [
    { value: "ALL", label: "All Tiers" },
    { value: "NORMAL", label: "Normal" },
    { value: "ELITE", label: "Elite" },
    { value: "BOSS", label: "Boss" },
];

// Descending by minWidth - mirrors the grid's `grid-cols-4 sm:6 md:8 lg:10`.
const COL_BREAKPOINTS = [
    { minWidth: 1024, cols: 10 },
    { minWidth: 768, cols: 8 },
    { minWidth: 640, cols: 6 },
    { minWidth: 0, cols: 4 },
];
const GRID_GAP_PX = 10; // gap-2.5
const CARD_TEXT_HEIGHT_PX = 30; // name row beneath the portrait
const ROW_ESTIMATE_PX = 150; // fallback before the grid width is measured

const SKELETON_GRID_IDS = Array.from({ length: 30 }, (_, i) => `enemy-skel-${i}`);

interface IEnemiesTabProps {
    /** `null` = profile private / unavailable; `undefined` = still loading. */
    encountered: IEncounteredEnemies | null | undefined;
    isLoading?: boolean;
}

export function EnemiesTab({ encountered, isLoading }: IEnemiesTabProps) {
    const { data: handbook, isLoading: isHandbookLoading } = useQuery(enemiesListQueryOptions());
    const [status, setStatus] = useState<StatusFilter>("all");
    const [level, setLevel] = useState<LevelFilter>("ALL");
    const [search, setSearch] = useState("");

    const encounteredIds = useMemo(() => new Set((encountered?.enemies ?? []).map((e) => e.enemyId)), [encountered]);

    // Displayable handbook = visible entries only, in Hypergryph's sortId order
    // (the query already sorts). Paired with whether the Doctor has seen each.
    const rows = useMemo<IEnemyRow[]>(() => (handbook ?? []).filter((e) => !e.hideInHandbook).map((enemy) => ({ enemy, seen: encounteredIds.has(enemy.enemyId) })), [handbook, encounteredIds]);

    const total = rows.length;
    const seenCount = useMemo(() => rows.reduce((n, r) => n + (r.seen ? 1 : 0), 0), [rows]);
    const pct = total > 0 ? Math.round((seenCount / total) * 100) : 0;

    const filtered = useMemo(() => {
        const q = search.trim().toLowerCase();
        return rows.filter(({ enemy, seen }) => {
            if (status === "encountered" && !seen) return false;
            if (status === "missing" && seen) return false;
            if (level !== "ALL" && enemy.enemyLevel !== level) return false;
            if (q && !enemy.name.toLowerCase().includes(q) && !enemy.enemyIndex.toLowerCase().includes(q)) return false;
            return true;
        });
    }, [rows, status, level, search]);

    if (isLoading || isHandbookLoading) {
        return (
            <section aria-busy="true" aria-label="Enemy handbook" className="flex flex-col gap-4">
                <Skeleton className="h-20 w-full rounded-2xl" />
                <div className="grid grid-cols-4 gap-2.5 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
                    {SKELETON_GRID_IDS.map((id) => (
                        <Skeleton className="aspect-3/4 w-full rounded-md" key={id} />
                    ))}
                </div>
            </section>
        );
    }

    if (encountered === null) {
        return <EnemiesEmpty body="This Doctor's enemy handbook is private, or their profile could not be found." title="Enemy data unavailable" />;
    }

    return (
        <section aria-label="Enemy handbook" className="flex flex-col gap-5">
            <ProgressHeader pct={pct} seen={seenCount} total={total} />

            <div className="flex w-full flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
                <InputGroup className="w-full sm:w-64 sm:min-w-48 sm:max-w-80 sm:flex-1">
                    <InputGroupAddon>
                        <Search />
                    </InputGroupAddon>
                    <InputGroupInput onChange={(e) => setSearch(e.target.value)} placeholder="Search enemies..." value={search} />
                </InputGroup>

                <div className="flex flex-wrap items-center gap-2">
                    <FilterChip active={status === "all"} count={total} label={STATUS_LABELS.all} onSelect={() => setStatus("all")} />
                    <FilterChip active={status === "encountered"} count={seenCount} label={STATUS_LABELS.encountered} onSelect={() => setStatus("encountered")} />
                    <FilterChip active={status === "missing"} count={total - seenCount} label={STATUS_LABELS.missing} onSelect={() => setStatus("missing")} />
                </div>

                <div className="flex flex-wrap items-center gap-2 sm:ml-auto">
                    {LEVEL_FILTERS.map((l) => (
                        <FilterChip active={level === l.value} key={l.value} label={l.label} onSelect={() => setLevel(l.value)} />
                    ))}
                </div>
            </div>

            {seenCount === 0 && total > 0 && <p className="rounded-xl border border-border/60 border-dashed bg-muted/30 px-4 py-3 text-muted-foreground text-sm">No enemies recorded yet. Encountered enemies populate on the next profile resync.</p>}

            {filtered.length === 0 ? <EnemiesEmpty body="Try a different filter or clear your search." title="No enemies match" /> : <VirtualizedEnemyGrid rows={filtered} />}
        </section>
    );
}

function getColumnCount(): number {
    if (typeof window === "undefined") return COL_BREAKPOINTS[COL_BREAKPOINTS.length - 1].cols;
    const w = window.innerWidth;
    for (const { minWidth, cols } of COL_BREAKPOINTS) {
        if (w >= minWidth) return cols;
    }
    return COL_BREAKPOINTS[COL_BREAKPOINTS.length - 1].cols;
}

/** Window-scroll virtualized grid: only the rows near the viewport are mounted,
 *  so the ~1.2k-card handbook stays smooth. Rows are uniform height, derived
 *  from the measured grid width, so no per-row measurement is needed. */
function VirtualizedEnemyGrid({ rows }: { rows: IEnemyRow[] }) {
    const parentRef = useRef<HTMLDivElement>(null);
    const [cols, setCols] = useState(getColumnCount);
    const [width, setWidth] = useState(0);
    const [scrollMargin, setScrollMargin] = useState(0);

    // Track column count + container width.
    useEffect(() => {
        const el = parentRef.current;
        if (!el) return;
        const onResize = () => setCols(getColumnCount());
        const observer = new ResizeObserver((entries) => {
            const w = entries[0]?.contentRect.width ?? 0;
            if (w > 0) setWidth(w);
        });
        observer.observe(el);
        window.addEventListener("resize", onResize, { passive: true });
        return () => {
            observer.disconnect();
            window.removeEventListener("resize", onResize);
        };
    }, []);

    // Distance from the document top, so window scrolling maps to the right rows.
    // Re-measured whenever layout above the grid can shift (width / row count).
    // biome-ignore lint/correctness/useExhaustiveDependencies: width/rows.length are re-measure triggers, not closure values.
    useLayoutEffect(() => {
        const measure = () => {
            if (!parentRef.current) return;
            const top = parentRef.current.getBoundingClientRect().top + window.scrollY;
            setScrollMargin((prev) => (Math.abs(prev - top) > 1 ? top : prev));
        };
        measure();
        window.addEventListener("resize", measure);
        return () => window.removeEventListener("resize", measure);
    }, [width, rows.length]);

    const chunks = useMemo(() => {
        const out: IEnemyRow[][] = [];
        for (let i = 0; i < rows.length; i += cols) out.push(rows.slice(i, i + cols));
        return out;
    }, [rows, cols]);

    const rowHeight = useMemo(() => {
        if (width <= 0) return ROW_ESTIMATE_PX;
        const cardWidth = (width - (cols - 1) * GRID_GAP_PX) / cols;
        return Math.round(cardWidth + CARD_TEXT_HEIGHT_PX + GRID_GAP_PX);
    }, [width, cols]);

    const virtualizer = useWindowVirtualizer({
        count: chunks.length,
        estimateSize: () => rowHeight,
        overscan: 3,
        scrollMargin,
    });

    // Re-flow rows when the derived height changes (width / column breakpoint).
    // biome-ignore lint/correctness/useExhaustiveDependencies: measure() must re-run when rowHeight changes.
    useLayoutEffect(() => virtualizer.measure(), [rowHeight, virtualizer]);

    const gridTemplateColumns = `repeat(${cols}, minmax(0, 1fr))`;

    return (
        <div ref={parentRef} style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
            {virtualizer.getVirtualItems().map((vi) => {
                const chunk = chunks[vi.index];
                if (!chunk) return null;
                return (
                    <div
                        key={vi.key}
                        style={{
                            contain: "content",
                            display: "grid",
                            gap: GRID_GAP_PX,
                            gridTemplateColumns,
                            left: 0,
                            position: "absolute",
                            top: 0,
                            transform: `translateY(${vi.start - scrollMargin}px)`,
                            width: "100%",
                        }}
                    >
                        {chunk.map(({ enemy, seen }) => (
                            <EnemyHandbookCard enemy={enemy} key={enemy.enemyId} seen={seen} />
                        ))}
                    </div>
                );
            })}
        </div>
    );
}

function ProgressHeader({ seen, total, pct }: { seen: number; total: number; pct: number }) {
    return (
        <div className="flex flex-col gap-2.5 rounded-2xl border border-border/50 bg-card/40 p-5">
            <div className="flex items-end justify-between gap-4">
                <div className="flex flex-col gap-1">
                    <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Enemy Handbook</span>
                    <div className="flex items-baseline gap-2">
                        <span className="font-bold text-2xl text-foreground tabular-nums leading-none">
                            {seen}
                            <span className="font-semibold text-lg text-muted-foreground"> / {total}</span>
                        </span>
                        <span className="text-muted-foreground text-sm">encountered</span>
                    </div>
                </div>
                <span className="font-mono font-semibold text-primary text-xl tabular-nums leading-none">{pct}%</span>
            </div>
            <div aria-label={`${pct}% of enemies encountered`} aria-valuemax={100} aria-valuemin={0} aria-valuenow={pct} className="h-2 w-full overflow-hidden rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_14%,transparent)]" role="progressbar">
                <div className="h-full rounded-full bg-primary transition-[width] duration-500 ease-out motion-reduce:transition-none" style={{ width: `${pct}%` }} />
            </div>
        </div>
    );
}

const EnemyHandbookCard = memo(function EnemyHandbookCard({ enemy, seen }: IEnemyRow) {
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait ? `${env.VITE_BACKEND_URL ?? ""}/api/assets${enemy.portrait}` : undefined;

    return (
        <div className="group relative flex flex-col overflow-hidden rounded-md border bg-card" style={{ borderColor: seen && enemy.enemyLevel !== "NORMAL" ? tok.accentSoft : "color-mix(in oklch, var(--border) 80%, transparent)" }} title={`${enemy.name} (${enemy.enemyIndex})${seen ? "" : " - not encountered"}`}>
            <div className="relative aspect-square w-full bg-[color-mix(in_oklch,var(--muted)_50%,transparent)]">
                {hasPortrait && portraitSrc ? (
                    // Not-yet-encountered enemies are desaturated/dimmed, but still visible.
                    <img alt={`${enemy.name} portrait`} className="block h-full w-full object-contain" decoding="async" loading="lazy" onError={() => setImgError(true)} src={portraitSrc} style={seen ? undefined : { filter: "grayscale(1)", opacity: 0.5 }} />
                ) : (
                    <EnemyPlaceholder className="h-full w-full p-4" />
                )}

                <span className="absolute top-1.5 left-1.5 rounded-sm bg-background/85 px-1 py-0.5 font-medium font-mono text-[8.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{enemy.enemyIndex}</span>

                {seen ? (
                    <span className="absolute top-1.5 right-1.5">
                        <LevelBadge level={enemy.enemyLevel} size="sm" />
                    </span>
                ) : (
                    <span className="absolute top-1.5 right-1.5 inline-flex rounded-sm bg-background/80 p-0.5" title="Not encountered">
                        <Lock aria-hidden className="size-3 text-muted-foreground" />
                    </span>
                )}

                {seen && enemy.enemyLevel !== "NORMAL" && <span aria-hidden className="absolute right-0 bottom-0 left-0 h-0.5" style={{ background: tok.accent }} />}
            </div>

            <div className="px-2 pt-1.5 pb-2">
                <h3 className={cn("m-0 line-clamp-1 font-sans font-semibold text-[11px] uppercase leading-tight tracking-[0.02em]", seen ? "text-foreground" : "text-muted-foreground")}>{enemy.name}</h3>
            </div>
        </div>
    );
});

interface IFilterChipProps {
    label: string;
    active: boolean;
    count?: number;
    onSelect: () => void;
}

function FilterChip({ label, active, count, onSelect }: IFilterChipProps) {
    return (
        <Toggle
            className={cn("h-8 gap-2 rounded-full px-3 font-medium text-[13px]", active && "border-primary/50 bg-primary/10 text-foreground shadow-[0_0_0_1px_color-mix(in_srgb,var(--primary)_25%,transparent),0_0_12px_color-mix(in_srgb,var(--primary)_20%,transparent)] hover:bg-primary/15")}
            onPressedChange={() => onSelect()}
            pressed={active}
            size="sm"
            variant="outline"
        >
            <span>{label}</span>
            {count != null && (
                <Badge className="font-mono tabular-nums" size="sm" variant={active ? "default" : "secondary"}>
                    {count}
                </Badge>
            )}
        </Toggle>
    );
}

function EnemiesEmpty({ title, body }: { title: string; body: string }) {
    return (
        <div className="flex flex-col items-center justify-center gap-3 rounded-2xl border border-border bg-card px-8 py-16 text-center">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Enemy Handbook</span>
            <h3 className="font-semibold text-lg tracking-tight">{title}</h3>
            <p className="max-w-sm text-muted-foreground text-sm">{body}</p>
        </div>
    );
}
