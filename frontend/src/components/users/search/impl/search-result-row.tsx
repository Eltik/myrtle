import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import type { SearchResultEntry } from "~/types/api";
import { getAvatarURL } from "../../leaderboard/impl/constants";
import { GradeBadge } from "../../leaderboard/impl/grade-badge";
import { UserHoverCard } from "./user-hover-card";

interface SearchResultRowProps {
    result: SearchResultEntry;
}

export const SearchResultRow = React.memo(function SearchResultRow({ result }: SearchResultRowProps) {
    // Only use fields available in the base search response
    // Account age and operator count require the massive data field, so we don't show them here
    const rowContent = (
        <Link href={`/user/${result.uid}`}>
            <div className="group flex items-center gap-4 rounded-lg border bg-card/50 p-3 transition-all duration-200 hover:border-primary/50 hover:bg-card">
                <Avatar className="h-12 w-12 shrink-0 border border-border">
                    <AvatarImage alt={result.nickname} src={getAvatarURL(result.avatarId) || "/placeholder.svg"} />
                    <AvatarFallback className="text-sm">{result.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h3 className="truncate font-medium transition-colors group-hover:text-primary">{result.nickname}</h3>
                        <GradeBadge grade={result.grade} size="sm" />
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
                    <p className="font-mono text-sm">{result.totalScore.toLocaleString()}</p>
                    <p className="text-muted-foreground text-xs">points</p>
                </div>

                {/* Mobile: show score */}
                <div className="shrink-0 text-right sm:hidden">
                    <p className="font-mono text-xs">{result.totalScore.toLocaleString()} pts</p>
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
