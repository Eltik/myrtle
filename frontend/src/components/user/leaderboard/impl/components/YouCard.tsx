import { Link } from "@tanstack/react-router";
import { Share2 } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import type { ILeaderboardEntry, IPlayerStanding } from "#/lib/api/user";
import { formatNumber, getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "../constants";
import { GradeBadge } from "./GradeBadge";

interface IYouCardProps {
    standing: IPlayerStanding | null;
    rankedDoctors: number | null;
}

export function YouCard({ standing, rankedDoctors }: IYouCardProps) {
    if (!standing) return null;
    const player: ILeaderboardEntry = standing.player;
    const nickname = player.nickname ?? `Doctor ${player.uid}`;
    const initials = nickname.slice(0, 2).toUpperCase();
    const avatarSrc = getAvatarById(player.avatar_id ?? DEFAULT_AVATAR_ID);
    const rank = player.rank_global ?? null;
    const percentileFromRank = rank != null && rankedDoctors && rankedDoctors > 0 ? (1 - rank / rankedDoctors) * 100 : null;
    const percentile = percentileFromRank ?? (1 - standing.percentile) * 100;
    const delta = standing.rank_delta;
    const grade = player.grade ?? "-";

    const deltaColor = delta == null ? "text-muted-foreground" : delta > 0 ? "text-success-foreground" : delta < 0 ? "text-destructive-foreground" : "text-muted-foreground";
    const deltaText = delta == null ? "-" : `${delta > 0 ? "+" : ""}${delta}`;

    const handleShare = async () => {
        try {
            await navigator.clipboard.writeText(`${window.location.origin}/user/${player.uid}`);
            toastManager.add({
                id: `lb-you-share-${Date.now()}`,
                title: "Profile link copied",
                description: "Your profile link is on the clipboard.",
                type: "success",
            });
        } catch {
            toastManager.add({
                id: `lb-you-share-error-${Date.now()}`,
                title: "Couldn't copy",
                description: "Clipboard access was denied.",
                type: "error",
            });
        }
    };

    return (
        <aside className="relative overflow-hidden rounded-xl border border-border bg-card p-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div aria-hidden className="pointer-events-none absolute inset-0" style={{ background: "radial-gradient(ellipse 80% 60% at 50% 0%, color-mix(in srgb, var(--primary) 8%, transparent), transparent 60%)" }} />
            <div className="relative mb-3 flex items-center justify-between font-mono text-[11px] font-medium uppercase leading-none tracking-[0.16em] text-muted-foreground">
                <span>Your standing</span>
                <span className="rounded-full border border-border bg-card px-2 py-0.5 text-foreground tabular-nums">#{rank ?? "-"}</span>
            </div>
            <div className="relative flex items-center gap-3">
                <Avatar className="size-12 rounded-2xl">
                    <AvatarImage src={avatarSrc} alt={nickname} />
                    <AvatarFallback className="rounded-2xl text-sm">{initials}</AvatarFallback>
                </Avatar>
                <div className="min-w-0">
                    <div className="truncate font-sans text-base font-bold leading-tight tracking-tight text-foreground">{nickname}</div>
                    <div className="mt-1 font-mono text-[11px] leading-none text-muted-foreground">
                        Rank <b className="font-bold text-foreground">#{rank ?? "-"}</b>
                        {percentile != null && (
                            <>
                                {" · Top "}
                                <b className="font-bold text-foreground">{percentile.toFixed(1)}%</b>
                            </>
                        )}
                    </div>
                </div>
            </div>
            <div className="relative mt-3.5 grid grid-cols-3 gap-2">
                <Metric k="Grade">
                    <GradeBadge grade={grade} className="size-7" />
                </Metric>
                <Metric k="Score">
                    <span className="font-sans text-base font-bold leading-none tracking-tight tabular-nums">{player.total_score == null ? "-" : formatNumber(player.total_score)}</span>
                </Metric>
                <Metric k="Δ Rank">
                    <span className={`font-sans text-base font-bold leading-none tracking-tight tabular-nums ${deltaColor}`}>{deltaText}</span>
                </Metric>
            </div>
            <div className="relative mt-3.5 flex items-center gap-2">
                <Button
                    variant="default"
                    size="sm"
                    className="flex-1"
                    render={
                        <Link to="/user/$id" params={{ id: player.uid }}>
                            View profile
                        </Link>
                    }
                />
                <Button variant="outline" size="icon-sm" aria-label="Share profile" onClick={handleShare}>
                    <Share2 />
                </Button>
            </div>
        </aside>
    );
}

function Metric({ k, children }: { k: string; children: React.ReactNode }) {
    return (
        <div className="flex flex-col gap-1.5 rounded-[10px] border border-border bg-muted/60 p-2.5">
            <span className="font-mono text-[9.5px] font-medium uppercase leading-none tracking-[0.14em] text-muted-foreground">{k}</span>
            <span className="flex items-center font-sans text-base font-bold leading-none tracking-tight tabular-nums text-foreground">{children}</span>
        </div>
    );
}
