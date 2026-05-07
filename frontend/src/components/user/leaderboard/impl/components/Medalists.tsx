import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { formatNumber, getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";
import type { LeaderboardEntry } from "../types";
import { GradeBadge } from "./GradeBadge";
import { ServerTag } from "./ServerTag";

interface IMedalistsProps {
    entries: LeaderboardEntry[];
    isLoading?: boolean;
}

const MEDAL: Record<number, { color: string; deep: string; label: string; tint: string }> = {
    1: { color: "#e8b04b", deep: "#a16a14", label: "Gold", tint: "color-mix(in srgb, #e8b04b 12%, var(--card))" },
    2: { color: "#c8c8c8", deep: "#7a7a7a", label: "Silver", tint: "color-mix(in srgb, #c8c8c8 14%, var(--card))" },
    3: { color: "#c87a3a", deep: "#7a4416", label: "Bronze", tint: "color-mix(in srgb, #c87a3a 12%, var(--card))" },
};

export function Medalists({ entries, isLoading }: IMedalistsProps) {
    return (
        <section className="rounded-xl border border-border bg-card px-4 pt-3.5 pb-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div className="mb-3 flex items-center justify-between">
                <span className="font-mono text-[11px] font-medium uppercase leading-none tracking-[0.16em] text-muted-foreground">Top 3 · overall</span>
                <span className="font-mono text-[10.5px] uppercase leading-none tracking-[0.14em] text-muted-foreground">Composite score</span>
            </div>
            <div className="flex flex-col gap-2">{entries.length > 0 ? entries.slice(0, 3).map((entry, index) => <MedalistRow key={entry.id} entry={entry} rank={index + 1} />) : [1, 2, 3].map((rank) => <MedalistSkeleton key={`skeleton-rank-${rank}`} rank={rank} loading={isLoading} />)}</div>
        </section>
    );
}

function MedalistRow({ entry, rank }: { entry: LeaderboardEntry; rank: number }) {
    const medal = MEDAL[rank];
    const nickname = entry.nickname ?? `Doctor ${entry.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(entry.avatar_id ?? DEFAULT_AVATAR_ID);

    return (
        <Link
            to="/user/$id"
            params={{ id: entry.uid }}
            className="group grid grid-cols-[44px_1fr] gap-3 overflow-hidden rounded-xl border border-border no-underline transition-colors hover:border-foreground/15"
            style={{
                background: `linear-gradient(90deg, ${medal.tint}, var(--card) 60%)`,
                borderColor: rank === 1 ? "color-mix(in srgb, #e8b04b 30%, var(--border))" : undefined,
            }}
        >
            <div
                role="img"
                aria-label={`${medal.label} medal · rank ${rank}`}
                className="flex items-center justify-center font-mono text-[22px] font-bold leading-none text-white shadow-[inset_0_1px_0_rgb(255_255_255/0.35),inset_0_-1.5px_0_rgb(0_0_0/0.18)]"
                style={{ background: `linear-gradient(180deg, ${medal.color}, ${medal.deep})`, textShadow: "0 1px 2px rgb(0 0 0 / 0.25)" }}
            >
                {rank}
            </div>
            <div className="grid grid-cols-[auto_1fr_auto] items-center gap-3 px-3 py-2.5 min-w-0">
                <Avatar className="size-11 rounded-xl">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-xl text-sm">{initials}</AvatarFallback>
                </Avatar>
                <div className="flex min-w-0 flex-col gap-1">
                    <span className="truncate font-sans text-sm font-bold leading-tight tracking-tight text-foreground">{nickname}</span>
                    <div className="flex items-center gap-1.5 font-mono text-[11px] leading-none text-muted-foreground">
                        <ServerTag server={entry.server} />
                        {entry.level != null && <span>Lv {entry.level}</span>}
                    </div>
                </div>
                <div className="flex items-center gap-2.5 font-mono tabular-nums">
                    <GradeBadge grade={entry.grade} className="size-8 text-sm" />
                    <span className="font-mono text-[17px] font-bold leading-none text-foreground tabular-nums">{entry.total_score == null ? "—" : formatNumber(entry.total_score)}</span>
                </div>
            </div>
        </Link>
    );
}

function MedalistSkeleton({ rank, loading }: { rank: number; loading?: boolean }) {
    const medal = MEDAL[rank];
    return (
        <div className="grid grid-cols-[44px_1fr] gap-3 overflow-hidden rounded-xl border border-border" style={{ background: `linear-gradient(90deg, ${medal.tint}, var(--card) 60%)` }}>
            <div className="flex items-center justify-center font-mono text-[22px] font-bold leading-none text-white shadow-[inset_0_1px_0_rgb(255_255_255/0.35),inset_0_-1.5px_0_rgb(0_0_0/0.18)]" style={{ background: `linear-gradient(180deg, ${medal.color}, ${medal.deep})` }}>
                {rank}
            </div>
            <div className="flex items-center gap-3 px-3 py-3.5">
                <span className="size-11 shrink-0 rounded-xl bg-muted" />
                <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                    <span className="h-3.5 w-32 rounded bg-muted" />
                    <span className="h-2.5 w-20 rounded bg-muted/70" />
                </div>
                <span className="font-mono text-[11px] uppercase leading-none tracking-[0.14em] text-muted-foreground">{loading ? "Loading…" : "—"}</span>
            </div>
        </div>
    );
}
