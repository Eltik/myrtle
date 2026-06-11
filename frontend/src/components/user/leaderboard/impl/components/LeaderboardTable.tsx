import { Link } from "@tanstack/react-router";
import { ChevronDown, ChevronUp, Minus } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { cn, getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID, formatPct, LEADERBOARD_SORTS, type LeaderboardSort, toPct } from "../constants";
import type { LeaderboardEntry } from "../types";
import { GradeBadge } from "./GradeBadge";
import { ServerTag } from "./ServerTag";

interface ILeaderboardTableProps {
    entries: LeaderboardEntry[];
    sort: LeaderboardSort;
    onSort: (next: LeaderboardSort) => void;
    isLoading?: boolean;
    intervalLabel?: string;
}

const DESKTOP_GRID = "grid-cols-[80px_minmax(0,1fr)_100px_90px_180px_60px]";

export function LeaderboardTable({ entries, sort, onSort, isLoading, intervalLabel = "since yesterday" }: ILeaderboardTableProps) {
    if (!isLoading && entries.length === 0) {
        return <div className="px-6 py-12 text-center font-sans text-muted-foreground text-sm">No Doctors match these filters.</div>;
    }

    return (
        <>
            <div className="hidden md:block">
                <div className={cn("grid items-center gap-4 border-border border-b bg-[color-mix(in_srgb,var(--muted)_35%,transparent)] px-4 py-3.5", DESKTOP_GRID)}>
                    <Th>Rank</Th>
                    <Th>Doctor</Th>
                    <Th>Server</Th>
                    <Th>Grade</Th>
                    <span className="flex justify-end">
                        <SortHeader sort={sort} onSort={onSort} />
                    </span>
                    <Th align="right">Lv</Th>
                </div>
                <ul className="contents">
                    {entries.map((entry) => (
                        <li key={entry.id} className="contents">
                            <DesktopRow entry={entry} sort={sort} intervalLabel={intervalLabel} />
                        </li>
                    ))}
                </ul>
            </div>
            <div className="block p-3 md:hidden">
                <div className="flex items-center justify-between px-1 pb-2">
                    <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]">Sort</span>
                    <SortHeader sort={sort} onSort={onSort} />
                </div>
                <div className="flex flex-col gap-2">
                    {entries.map((entry) => (
                        <MobileRow key={entry.id} entry={entry} sort={sort} intervalLabel={intervalLabel} />
                    ))}
                </div>
            </div>
        </>
    );
}

function SortHeader({ sort, onSort }: { sort: LeaderboardSort; onSort: (next: LeaderboardSort) => void }) {
    const activeLabel = LEADERBOARD_SORTS.find((s) => s.value === sort)?.label ?? "Total";
    return (
        <DropdownMenu>
            <DropdownMenuTrigger className="inline-flex cursor-pointer items-center gap-1 whitespace-nowrap font-medium font-mono text-[11px] text-foreground uppercase leading-none tracking-[0.16em] transition-colors hover:text-primary" aria-label="Change sort category">
                {activeLabel}
                <ChevronDown className="size-3 opacity-70" />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="min-w-40">
                {LEADERBOARD_SORTS.map((opt) => (
                    <DropdownMenuItem key={opt.value} onClick={() => onSort(opt.value)} className={cn("cursor-pointer", sort === opt.value && "font-semibold text-primary")}>
                        {opt.label}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

function RankMovement({ delta, intervalLabel = "since yesterday" }: { delta: number | null; intervalLabel?: string }) {
    if (delta == null) return null;
    if (delta === 0) {
        return (
            <span className="inline-flex items-center gap-0.5 font-mono font-semibold text-[10px] text-muted-foreground tabular-nums leading-none" title={`No change in rank ${intervalLabel}`}>
                <Minus className="size-2.5" aria-hidden /> 0
            </span>
        );
    }
    const isUp = delta > 0;
    const Icon = isUp ? ChevronUp : ChevronDown;
    const color = isUp ? "text-success-foreground" : "text-destructive-foreground";
    const label = `${isUp ? "Climbed" : "Fell"} ${Math.abs(delta)} ${Math.abs(delta) === 1 ? "rank" : "ranks"} ${intervalLabel}`;
    return (
        <span className={cn("inline-flex items-center gap-0.5 font-mono font-semibold text-[10px] tabular-nums leading-none", color)} title={label}>
            <Icon className="size-2.5" aria-hidden /> {Math.abs(delta)}
        </span>
    );
}

function Th({ children, align, sorted }: { children: React.ReactNode; align?: "right"; sorted?: boolean }) {
    return <span className={cn("whitespace-nowrap font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]", align === "right" ? "text-right" : "text-left", sorted && "text-foreground")}>{children}</span>;
}

function DesktopRow({ entry, sort, intervalLabel }: { entry: LeaderboardEntry; sort: LeaderboardSort; intervalLabel?: string }) {
    const nickname = entry.nickname ?? `Doctor ${entry.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(entry.avatar_id ?? DEFAULT_AVATAR_ID);
    const score = entry[sort];
    const ratio = toPct(score);

    const rowBg = entry.isSelf ? "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)] hover:bg-[color-mix(in_srgb,var(--primary)_10%,transparent)]" : "hover:bg-[color-mix(in_srgb,var(--accent)_50%,transparent)]";

    return (
        <Link to="/user/$id" params={{ id: entry.uid }} aria-label={`View ${nickname} profile`} className={cn("group grid items-center gap-4 border-[color-mix(in_srgb,var(--border)_60%,transparent)] border-b px-4 py-3 font-sans text-[13px] text-foreground no-underline transition-colors", DESKTOP_GRID, rowBg)}>
            <span className="inline-flex min-w-14 flex-col items-start gap-1 font-mono font-semibold text-foreground text-sm tabular-nums leading-none">
                <span>#{entry.rank_global ?? "-"}</span>
                <RankMovement delta={entry.rank_delta} intervalLabel={intervalLabel} />
            </span>
            <span className="inline-flex min-w-0 max-w-80 items-center gap-3">
                <Avatar className="size-9 rounded-[10px]">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-[10px] text-xs">{initials}</AvatarFallback>
                </Avatar>
                <span className="flex min-w-0 flex-col gap-0.5">
                    <span className="truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight transition-colors group-hover:text-primary">{nickname}</span>
                    <span className="truncate font-mono text-[11px] text-muted-foreground tabular-nums leading-none before:opacity-50 before:content-['#']">{entry.uid}</span>
                </span>
            </span>
            <span>
                <ServerTag server={entry.server} />
            </span>
            <span>
                <GradeBadge grade={entry.grade} />
            </span>
            <span className="text-right">
                <span className="ml-auto inline-flex min-w-24 flex-col items-end gap-1.5">
                    <span className="font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">{score == null ? "-" : formatPct(score)}</span>
                    <span className="relative block h-0.75 w-full overflow-hidden rounded-full bg-muted">
                        <span className="absolute inset-y-0 left-0 rounded-full bg-linear-to-r from-primary/70 to-primary" style={{ width: `${ratio}%` }} />
                    </span>
                </span>
            </span>
            <span className="text-right font-mono text-xs tabular-nums">{entry.level == null ? "-" : entry.level}</span>
        </Link>
    );
}

function MobileRow({ entry, sort, intervalLabel }: { entry: LeaderboardEntry; sort: LeaderboardSort; intervalLabel?: string }) {
    const nickname = entry.nickname ?? `Doctor ${entry.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(entry.avatar_id ?? DEFAULT_AVATAR_ID);

    return (
        <Link to="/user/$id" params={{ id: entry.uid }} className={cn("grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-xl border border-border bg-card px-3 py-3 no-underline transition-colors hover:border-foreground/15", entry.isSelf && "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)]")}>
            <span className="inline-flex flex-col items-center gap-1">
                <span className="inline-flex size-9 items-center justify-center rounded-[10px] bg-muted font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">#{entry.rank_global ?? "-"}</span>
                <RankMovement delta={entry.rank_delta} intervalLabel={intervalLabel} />
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
                    <ServerTag server={entry.server} />
                    {entry.level != null && <span>· Lv {entry.level}</span>}
                </div>
            </div>
            <div className="flex flex-col items-end gap-1.5">
                <GradeBadge grade={entry.grade} />
                <span className="font-bold font-mono text-foreground text-sm tabular-nums leading-none tracking-tight">{entry[sort] == null ? "-" : formatPct(entry[sort])}</span>
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
