import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import type { SearchResultEntry } from "~/types/api";
import { getAvatarUrl } from "../../leaderboard/impl/constants";
import { GradeBadge } from "../../leaderboard/impl/grade-badge";
import { formatRelativeTime } from "./helpers";

interface SearchResultRowProps {
    result: SearchResultEntry;
}

export const SearchResultRow = React.memo(function SearchResultRow({ result }: SearchResultRowProps) {
    return (
        <Link href={`/user/${result.uid}`}>
            <div className="group flex items-center gap-4 rounded-lg border bg-card/50 p-3 transition-all duration-200 hover:border-primary/50 hover:bg-card">
                <Avatar className="h-12 w-12 shrink-0 border border-border">
                    <AvatarImage alt={result.nickname} src={getAvatarUrl(result.avatarId) || "/placeholder.svg"} />
                    <AvatarFallback className="text-sm">{result.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                </Avatar>

                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h3 className="truncate font-medium transition-colors group-hover:text-primary">{result.nickname}</h3>
                        <Badge className="shrink-0 text-xs uppercase" variant="secondary">
                            {result.server}
                        </Badge>
                    </div>
                    <p className="text-muted-foreground text-sm">
                        Level {result.level} · UID: {result.uid}
                    </p>
                </div>

                <div className="hidden items-center gap-4 sm:flex">
                    <div className="text-right">
                        <p className="text-muted-foreground text-xs">Updated</p>
                        <p className="text-sm">{formatRelativeTime(result.updatedAt)}</p>
                    </div>
                    <GradeBadge grade={result.grade} />
                </div>

                <div className="flex items-center sm:hidden">
                    <GradeBadge grade={result.grade} size="sm" />
                </div>
            </div>
        </Link>
    );
});
