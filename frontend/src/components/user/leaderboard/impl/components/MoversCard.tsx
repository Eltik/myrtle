import { Link } from "@tanstack/react-router";
import { ChevronUp } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import type { ILeaderboardMover } from "#/lib/api/user";
import { getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";

export function MoversCard({ movers, isLoading }: { movers: ILeaderboardMover[]; isLoading?: boolean }) {
    return (
        <aside className="rounded-xl border border-border bg-card p-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div className="mb-2.5 flex items-center justify-between">
                <span className="font-sans text-[13.5px] font-semibold leading-tight tracking-tight text-foreground">Top movers · today</span>
                <span className="font-mono text-[10px] font-medium uppercase leading-none tracking-[0.14em] text-muted-foreground">Δ Rank</span>
            </div>
            <div className="flex flex-col gap-2.5">
                {isLoading && movers.length === 0 ? (
                    ["a", "b", "c"].map((slot) => <MoverSkeleton key={`mover-skeleton-${slot}`} />)
                ) : movers.length === 0 ? (
                    <span className="font-mono text-[11px] text-muted-foreground">No movements yet.</span>
                ) : (
                    movers.slice(0, 3).map((mover) => <MoverRow key={mover.uid} mover={mover} />)
                )}
            </div>
        </aside>
    );
}

function MoverRow({ mover }: { mover: ILeaderboardMover }) {
    const nickname = mover.nickname ?? `Doctor ${mover.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(mover.avatar_id ?? DEFAULT_AVATAR_ID);

    return (
        <Link to="/user/$id" params={{ id: mover.uid }} className="flex items-center gap-2.5 no-underline">
            <Avatar className="size-8 rounded-lg">
                <AvatarImage src={avatarSrc} alt={nickname} />
                <AvatarFallback className="rounded-lg text-[10px]">{initials}</AvatarFallback>
            </Avatar>
            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="truncate font-sans text-[12.5px] font-semibold leading-tight text-foreground">{nickname}</span>
                <span className="font-mono text-[11px] leading-none text-muted-foreground tabular-nums">
                    #{mover.current_rank} · {mover.server.toUpperCase()}
                </span>
            </div>
            <span className="inline-flex items-center gap-0.5 font-sans text-[13px] font-bold leading-none tracking-tight text-success-foreground tabular-nums">
                <ChevronUp className="size-3" /> {mover.rank_delta}
            </span>
        </Link>
    );
}

function MoverSkeleton() {
    return (
        <div className="flex items-center gap-2.5">
            <span className="size-8 rounded-lg bg-muted" />
            <div className="flex flex-1 flex-col gap-1">
                <span className="h-3 w-28 rounded bg-muted" />
                <span className="h-2.5 w-16 rounded bg-muted/70" />
            </div>
            <span className="h-3 w-8 rounded bg-muted" />
        </div>
    );
}
