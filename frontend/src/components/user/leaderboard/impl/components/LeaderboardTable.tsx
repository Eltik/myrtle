import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronUp, Minus } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import type { IMaterials } from "#/lib/api/materials";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { formatPct, type LeaderboardMessageKey, type Ranking, toPct } from "../constants";
import type { messages as constantsMessages } from "../constants.messages";
import { playerIdentity } from "../identity";
import type { ICatalog } from "../inventory.types";
import type { IRankedRow } from "../types";
import { GradeBadge } from "./GradeBadge";
import type { messages } from "./LeaderboardTable.messages";
import { RankByPicker } from "./RankByPicker";
import { ServerTag } from "./ServerTag";

/** The option labels in `constants.messages.ts` are rendered here too. */
type TableT = TypedT<typeof messages & typeof constantsMessages>;

interface ILeaderboardTableProps {
    rows: IRankedRow[];
    ranking: Ranking;
    onRanking: (next: Ranking) => void;
    catalog: ICatalog;
    materials: IMaterials | undefined;
    /** For an item ranking, the largest holding; every row's bar is drawn against it. */
    topValue: number | null;
    isLoading?: boolean;
    intervalKey?: LeaderboardMessageKey;
}

// Tighter fixed columns on tablets so the player name keeps ~200px at 768px;
// the roomier tier from lg up is the original.
const DESKTOP_GRID = "grid-cols-[72px_minmax(0,1fr)_88px_76px_150px_52px] lg:grid-cols-[80px_minmax(0,1fr)_100px_90px_180px_60px]";

/** Bar fill for a value: a score is already 0..1, a quantity is a share of the top holding. */
function fill(ranking: Ranking, value: number | null, topValue: number | null): number {
    if (value == null) return 0;
    if (ranking.kind === "score") return toPct(value);
    if (!topValue || topValue <= 0) return 0;
    return Math.max(0, Math.min(100, (value / topValue) * 100));
}

export function LeaderboardTable({ rows, ranking, onRanking, catalog, materials, topValue, isLoading, intervalKey = "leaderboard.interval.day.since" }: ILeaderboardTableProps) {
    const t: TableT = useT("user");

    if (!isLoading && rows.length === 0) {
        return <div className="px-6 py-12 text-center font-sans text-muted-foreground text-sm">{ranking.kind === "item" ? t("leaderboard.table.empty.item") : t("leaderboard.table.empty")}</div>;
    }

    const picker = <RankByPicker ranking={ranking} onRanking={onRanking} catalog={catalog} materials={materials} variant="header" />;

    return (
        <>
            <div className="hidden md:block">
                <div className={cn("grid items-center gap-4 border-border border-b bg-[color-mix(in_srgb,var(--muted)_35%,transparent)] px-4 py-3.5", DESKTOP_GRID)}>
                    <Th>{t("leaderboard.table.th.rank")}</Th>
                    <Th>{t("leaderboard.table.th.doctor")}</Th>
                    <Th>{t("leaderboard.table.th.server")}</Th>
                    <Th>{t("leaderboard.table.th.grade")}</Th>
                    <span className="flex min-w-0 justify-end">{picker}</span>
                    <Th align="right">{t("leaderboard.table.th.level")}</Th>
                </div>
                <ul className="contents">
                    {rows.map((row) => (
                        <li key={`${row.server}:${row.uid}`} className="contents">
                            <DesktopRow row={row} ranking={ranking} topValue={topValue} intervalKey={intervalKey} />
                        </li>
                    ))}
                </ul>
            </div>
            {/* No header row on phones: the toolbar's "Rank by" control sits
                directly above the cards, so a second one here was a duplicate. */}
            <div className="block p-3 md:hidden">
                <div className="flex flex-col gap-2">
                    {rows.map((row) => (
                        <MobileRow key={`${row.server}:${row.uid}`} row={row} ranking={ranking} topValue={topValue} intervalKey={intervalKey} />
                    ))}
                </div>
            </div>
        </>
    );
}

function RankMovement({ delta, intervalKey = "leaderboard.interval.day.since" }: { delta: number | null; intervalKey?: LeaderboardMessageKey }) {
    const t: TableT = useT("user");

    if (delta == null) return null;
    if (delta === 0) {
        return (
            <span className="inline-flex items-center gap-0.5 font-mono font-semibold text-[10px] text-muted-foreground tabular-nums leading-none" title={t("leaderboard.table.movement.none", { interval: t(intervalKey) })}>
                <Minus className="size-2.5" aria-hidden /> 0
            </span>
        );
    }
    const isUp = delta > 0;
    const Icon = isUp ? ChevronUp : ChevronDown;
    const color = isUp ? "text-success-foreground" : "text-destructive-foreground";
    const label = t(isUp ? "leaderboard.table.movement.up" : "leaderboard.table.movement.down", { count: Math.abs(delta), interval: t(intervalKey) });
    return (
        <span className={cn("inline-flex items-center gap-0.5 font-mono font-semibold text-[10px] tabular-nums leading-none", color)} title={label}>
            <Icon className="size-2.5" aria-hidden /> {Math.abs(delta)}
        </span>
    );
}

function Th({ children, align }: { children: React.ReactNode; align?: "right" }) {
    return <span className={cn("whitespace-nowrap font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]", align === "right" ? "text-right" : "text-left")}>{children}</span>;
}

/** The ranked value with its bar: a percentage for a score, a count for an item. */
function ValueCell({ row, ranking, topValue, className }: { row: IRankedRow; ranking: Ranking; topValue: number | null; className?: string }) {
    const t: TableT = useT("user");
    const f = useFormatters();
    const pct = fill(ranking, row.value, topValue);
    const text = row.value == null ? "-" : ranking.kind === "score" ? formatPct(row.value) : f.number(row.value);
    const title = ranking.kind === "item" && row.value != null ? t("leaderboard.table.share", { pct: Math.round(pct) }) : undefined;
    return (
        <span className={cn("inline-flex min-w-24 flex-col items-end gap-1.5", className)} title={title}>
            <span className="font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">{text}</span>
            <span className="relative block h-0.75 w-full overflow-hidden rounded-full bg-muted">
                <span className="absolute inset-y-0 left-0 rounded-full bg-linear-to-r from-primary/70 to-primary transition-[width] duration-300 motion-reduce:transition-none" style={{ width: `${pct}%` }} />
            </span>
        </span>
    );
}

function DesktopRow({ row, ranking, topValue, intervalKey }: { row: IRankedRow; ranking: Ranking; topValue: number | null; intervalKey?: LeaderboardMessageKey }) {
    const t: TableT = useT("user");
    const { nickname, initials, avatarSrc } = playerIdentity(row);

    const rowBg = row.isSelf ? "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)] hover:bg-[color-mix(in_srgb,var(--primary)_10%,transparent)]" : "hover:bg-[color-mix(in_srgb,var(--accent)_50%,transparent)]";

    return (
        <Link
            to="/user/$id"
            params={{ id: row.uid }}
            aria-label={t("leaderboard.table.viewProfile", { nickname })}
            className={cn("group grid items-center gap-4 border-[color-mix(in_srgb,var(--border)_60%,transparent)] border-b px-4 py-3 font-sans text-[13px] text-foreground no-underline transition-colors", DESKTOP_GRID, rowBg)}
        >
            <span className="inline-flex min-w-14 flex-col items-start gap-1 font-mono font-semibold text-foreground text-sm tabular-nums leading-none">
                <span>#{row.rank ?? "-"}</span>
                <RankMovement delta={row.delta} intervalKey={intervalKey} />
            </span>
            <span className="inline-flex min-w-0 max-w-80 items-center gap-3">
                <Avatar className="size-9 rounded-[10px]">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-[10px] text-xs">{initials}</AvatarFallback>
                </Avatar>
                <span className="flex min-w-0 flex-col gap-0.5">
                    <span className="truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight transition-colors group-hover:text-primary">{nickname}</span>
                    <span className="truncate font-mono text-[11px] text-muted-foreground tabular-nums leading-none before:opacity-50 before:content-['#']">{row.uid}</span>
                </span>
            </span>
            <span>
                <ServerTag server={row.server} />
            </span>
            <span>
                <GradeBadge grade={row.grade} />
            </span>
            <span className="text-right">
                <ValueCell row={row} ranking={ranking} topValue={topValue} className="ml-auto" />
            </span>
            <span className="text-right font-mono text-xs tabular-nums">{row.level == null ? "-" : row.level}</span>
        </Link>
    );
}

function MobileRow({ row, ranking, topValue, intervalKey }: { row: IRankedRow; ranking: Ranking; topValue: number | null; intervalKey?: LeaderboardMessageKey }) {
    const t: TableT = useT("user");
    const { nickname, initials, avatarSrc } = playerIdentity(row);

    return (
        <Link to="/user/$id" params={{ id: row.uid }} className={cn("grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-xl border border-border bg-card px-3 py-3 no-underline transition-colors hover:border-foreground/15", row.isSelf && "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)]")}>
            <span className="inline-flex flex-col items-center gap-1">
                <span className="inline-flex size-9 items-center justify-center rounded-[10px] bg-muted font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">#{row.rank ?? "-"}</span>
                <RankMovement delta={row.delta} intervalKey={intervalKey} />
            </span>
            <div className="flex min-w-0 flex-col gap-1">
                <div className="flex min-w-0 items-center gap-2">
                    <Avatar className="size-7 rounded-lg">
                        <AvatarImage src={avatarSrc} alt={nickname} />
                        <AvatarFallback className="rounded-lg text-[10px]">{initials}</AvatarFallback>
                    </Avatar>
                    <span className="truncate font-sans font-semibold text-foreground text-sm leading-tight tracking-tight">{nickname}</span>
                </div>
                <div className="flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground leading-none">
                    <ServerTag server={row.server} />
                    {row.level != null && <span>{t("leaderboard.table.levelInline", { level: row.level })}</span>}
                </div>
            </div>
            <div className="flex flex-col items-end gap-1.5">
                <GradeBadge grade={row.grade} />
                <ValueCell row={row} ranking={ranking} topValue={topValue} className="min-w-20" />
            </div>
        </Link>
    );
}

export function LeaderboardTableSkeleton({ rows = 8 }: { rows?: number }) {
    return (
        <>
            <div className="hidden md:block">
                <div className="px-4 pt-3 pb-1">
                    <div className={cn("grid h-9 items-center gap-4 border-border border-b", DESKTOP_GRID)} />
                </div>
                <div className="divide-y divide-border/60">
                    {Array.from({ length: rows }, (_, i) => `row-skeleton-${i}-of-${rows}`).map((key) => (
                        <div key={key} className={cn("grid items-center gap-4 px-4 py-3", DESKTOP_GRID)}>
                            <span className="h-4 w-12 rounded bg-muted" />
                            <div className="flex items-center gap-3">
                                <span className="size-9 rounded-[10px] bg-muted" />
                                <div className="flex flex-col gap-1.5">
                                    <span className="h-3 w-32 rounded bg-muted" />
                                    <span className="h-2.5 w-20 rounded bg-muted/70" />
                                </div>
                            </div>
                            <span className="h-5 w-10 rounded bg-muted" />
                            <span className="h-7 w-7 rounded-md bg-muted" />
                            <span className="h-3 w-full rounded bg-muted" />
                            <span className="h-3 w-8 rounded bg-muted" />
                        </div>
                    ))}
                </div>
            </div>
            <div className="block p-3 md:hidden">
                <div className="flex flex-col gap-2">
                    {Array.from({ length: Math.min(rows, 6) }, (_, i) => `mobile-skeleton-${i}`).map((key) => (
                        <div key={key} className="grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-xl border border-border bg-card px-3 py-3">
                            <span className="size-9 rounded-[10px] bg-muted" />
                            <div className="flex flex-col gap-1.5">
                                <span className="h-3 w-32 rounded bg-muted" />
                                <span className="h-2.5 w-20 rounded bg-muted/70" />
                            </div>
                            <span className="h-7 w-10 rounded bg-muted" />
                        </div>
                    ))}
                </div>
            </div>
        </>
    );
}
