import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import type { UserProfile } from "~/types/api";
import { getAvatarURL } from "../../leaderboard/impl/constants";
import { GradeBadge } from "../../leaderboard/impl/grade-badge";
import { UserHoverCard } from "./user-hover-card";

interface SearchResultRowProps {
    result: UserProfile;
}

export const SearchResultRow = React.memo(function SearchResultRow({ result }: SearchResultRowProps) {
    const nickname = result.nickname ?? "Unknown";
    const rowContent = (
        <Link href={`/user/${result.uid}`}>
            <div className="group flex items-center gap-4 rounded-lg border bg-card/50 p-3 transition-all duration-200 hover:border-primary/50 hover:bg-card">
                <Avatar className="h-12 w-12 shrink-0 border border-border">
                    <AvatarImage alt={nickname} src={getAvatarURL(result.avatar_id) || "/placeholder.svg"} />
                    <AvatarFallback className="text-sm">{nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h3 className="truncate font-medium transition-colors group-hover:text-primary">{nickname}</h3>
                        <GradeBadge grade={result.grade ?? "F"} size="sm" />
                        <Badge className="shrink-0 border border-border bg-secondary/80 text-xs uppercase" variant="secondary">
                            {result.server}
                        </Badge>
                    </div>
                    <p className="text-muted-foreground text-sm">
                        Level {result.level} · UID: {result.uid}
                    </p>
                </div>

                {/* Score display */}
                <div className="hidden shrink-0 text-right sm:block">
                    <p className="font-mono text-sm">{(result.total_score ?? 0).toLocaleString()}</p>
                    <p className="text-muted-foreground text-xs">points</p>
                </div>

                {/* Mobile: show score */}
                <div className="shrink-0 text-right sm:hidden">
                    <p className="font-mono text-xs">{(result.total_score ?? 0).toLocaleString()} pts</p>
                </div>
            </div>
        </Link>
    );

    return (
        <UserHoverCard result={result} side="top">
            {rowContent}
        </UserHoverCard>
    );
});
