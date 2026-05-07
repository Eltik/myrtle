import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { cn, formatNumber, getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";
import type { LeaderboardEntry } from "../types";
import { GradeBadge } from "./GradeBadge";
import { ServerTag } from "./ServerTag";

interface ILeaderboardTableProps {
    entries: LeaderboardEntry[];
    referenceScore: number | null;
    isLoading?: boolean;
}

export function LeaderboardTable({ entries, referenceScore, isLoading }: ILeaderboardTableProps) {
    if (!isLoading && entries.length === 0) {
        return <div className="px-6 py-12 text-center font-sans text-sm text-muted-foreground">No Doctors match these filters.</div>;
    }

    return (
        <>
            <div className="hidden overflow-x-auto md:block">
                <table className="w-full border-separate border-spacing-0">
                    <thead>
                        <tr>
                            <Th>Rank</Th>
                            <Th>Doctor</Th>
                            <Th>Server</Th>
                            <Th>Grade</Th>
                            <Th align="right" sorted>
                                Score <span className="ml-1 text-[10px] text-primary">▼</span>
                            </Th>
                            <Th align="right">Lv</Th>
                        </tr>
                    </thead>
                    <tbody>
                        {entries.map((entry) => (
                            <DesktopRow key={entry.id} entry={entry} referenceScore={referenceScore} />
                        ))}
                    </tbody>
                </table>
            </div>
            <div className="block p-3 md:hidden">
                <div className="flex flex-col gap-2">
                    {entries.map((entry) => (
                        <MobileRow key={entry.id} entry={entry} />
                    ))}
                </div>
            </div>
        </>
    );
}

function Th({ children, align, sorted }: { children: React.ReactNode; align?: "right"; sorted?: boolean }) {
    return (
        <th
            scope="col"
            className={cn("border-b border-border bg-[color-mix(in_srgb,var(--muted)_35%,transparent)] px-4 py-3.5 font-mono text-[11px] font-medium uppercase leading-none tracking-[0.16em] text-muted-foreground whitespace-nowrap", align === "right" ? "text-right" : "text-left", sorted && "text-foreground")}
        >
            {children}
        </th>
    );
}

function DesktopRow({ entry, referenceScore }: { entry: LeaderboardEntry; referenceScore: number | null }) {
    const nickname = entry.nickname ?? `Doctor ${entry.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(entry.avatar_id ?? DEFAULT_AVATAR_ID);
    const score = entry.total_score;
    const ratio = score == null || referenceScore == null || referenceScore <= 0 ? 0 : Math.max(0, Math.min(100, (score / referenceScore) * 100));

    const cellBg = entry.isSelf ? "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)] hover:bg-[color-mix(in_srgb,var(--primary)_10%,transparent)]" : "hover:bg-[color-mix(in_srgb,var(--accent)_50%,transparent)]";
    const cellCls = cn("border-b border-[color-mix(in_srgb,var(--border)_60%,transparent)] px-4 py-3 align-middle font-sans text-[13px] text-foreground", cellBg);

    return (
        <tr className="group cursor-pointer transition-colors">
            <td className={cellCls}>
                <Link to="/user/$id" params={{ id: entry.uid }} className="inline-flex min-w-14 items-center font-mono text-sm font-semibold leading-none text-foreground tabular-nums no-underline">
                    #{entry.rank_global ?? "—"}
                </Link>
            </td>
            <td className={cellCls}>
                <Link to="/user/$id" params={{ id: entry.uid }} className="inline-flex max-w-80 min-w-0 items-center gap-3 no-underline">
                    <Avatar className="size-9 rounded-[10px]">
                        <AvatarImage src={avatarSrc} alt={nickname} />
                        <AvatarFallback className="rounded-[10px] text-xs">{initials}</AvatarFallback>
                    </Avatar>
                    <div className="flex min-w-0 flex-col gap-0.5">
                        <span className="truncate font-sans text-[13.5px] font-semibold leading-tight tracking-tight text-foreground transition-colors group-hover:text-primary">{nickname}</span>
                        <span className="truncate font-mono text-[11px] leading-none text-muted-foreground tabular-nums before:opacity-50 before:content-['#']">{entry.uid}</span>
                    </div>
                </Link>
            </td>
            <td className={cellCls}>
                <ServerTag server={entry.server} />
            </td>
            <td className={cellCls}>
                <GradeBadge grade={entry.grade} />
            </td>
            <td className={cn(cellCls, "text-right")}>
                <div className="ml-auto inline-flex min-w-24 flex-col items-end gap-1.5">
                    <span className="font-mono text-[13px] font-semibold leading-none text-foreground tabular-nums">{score == null ? "—" : formatNumber(score)}</span>
                    <span className="relative block h-0.75 w-full overflow-hidden rounded-full bg-muted">
                        <span className="absolute inset-y-0 left-0 rounded-full bg-linear-to-r from-primary/70 to-primary" style={{ width: `${ratio}%` }} />
                    </span>
                </div>
            </td>
            <td className={cn(cellCls, "text-right font-mono text-xs tabular-nums")}>{entry.level == null ? "—" : entry.level}</td>
        </tr>
    );
}

function MobileRow({ entry }: { entry: LeaderboardEntry }) {
    const nickname = entry.nickname ?? `Doctor ${entry.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(entry.avatar_id ?? DEFAULT_AVATAR_ID);

    return (
        <Link to="/user/$id" params={{ id: entry.uid }} className={cn("grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-xl border border-border bg-card px-3 py-3 no-underline transition-colors hover:border-foreground/15", entry.isSelf && "bg-[color-mix(in_srgb,var(--primary)_6%,transparent)]")}>
            <span className="inline-flex size-9 items-center justify-center rounded-[10px] bg-muted font-mono text-[13px] font-semibold leading-none text-foreground tabular-nums">#{entry.rank_global ?? "—"}</span>
            <div className="flex min-w-0 flex-col gap-1">
                <div className="flex min-w-0 items-center gap-2">
                    <Avatar className="size-7 rounded-lg">
                        <AvatarImage src={avatarSrc} alt={nickname} />
                        <AvatarFallback className="rounded-lg text-[10px]">{initials}</AvatarFallback>
                    </Avatar>
                    <span className="truncate font-sans text-sm font-semibold leading-tight tracking-tight text-foreground">{nickname}</span>
                </div>
                <div className="flex items-center gap-1.5 font-mono text-[11px] leading-none text-muted-foreground">
                    <ServerTag server={entry.server} />
                    {entry.level != null && <span>· Lv {entry.level}</span>}
                </div>
            </div>
            <div className="flex flex-col items-end gap-1.5">
                <GradeBadge grade={entry.grade} />
                <span className="font-mono text-sm font-bold leading-none tracking-tight tabular-nums text-foreground">{entry.total_score == null ? "—" : formatNumber(entry.total_score)}</span>
            </div>
        </Link>
    );
}

export function LeaderboardTableSkeleton({ rows = 8 }: { rows?: number }) {
    return (
        <>
            <div className="hidden md:block">
                <div className="px-4 pt-3 pb-1">
                    <div className="grid h-9 grid-cols-[80px_1fr_80px_60px_140px_60px] items-center gap-4 border-b border-border" />
                </div>
                <div className="divide-y divide-border/60">
                    {Array.from({ length: rows }, (_, i) => `row-skeleton-${i}-of-${rows}`).map((key) => (
                        <div key={key} className="grid grid-cols-[80px_1fr_80px_60px_140px_60px] items-center gap-4 px-4 py-3">
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
