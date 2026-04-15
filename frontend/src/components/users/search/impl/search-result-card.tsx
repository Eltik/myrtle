import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { UserProfile } from "~/types/api";
import { getAvatarURL } from "../../leaderboard/impl/constants";
import { GradeBadge } from "../../leaderboard/impl/grade-badge";
import { UserHoverCard } from "./user-hover-card";

interface SearchResultCardProps {
    result: UserProfile;
}

export const SearchResultCard = React.memo(function SearchResultCard({ result }: SearchResultCardProps) {
    const nickname = result.nickname ?? "Unknown";
    const cardContent = (
        <Link href={`/user/${result.uid}`}>
            <Card className="group h-full overflow-hidden py-0 transition-all duration-200 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/5">
                <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                        <Avatar className="h-14 w-14 shrink-0 border border-border transition-transform duration-200 group-hover:scale-105">
                            <AvatarImage alt={nickname} src={getAvatarURL(result.avatar_id) || "/placeholder.svg"} />
                            <AvatarFallback className="text-sm">{nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                        </Avatar>
                        <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                                <h3 className="truncate font-medium text-base transition-colors group-hover:text-primary">{nickname}</h3>
                                <GradeBadge grade={result.grade ?? "F"} size="sm" />
                            </div>
                            <div className="mt-1 flex flex-wrap items-center gap-1.5">
                                <Badge className="border border-border bg-secondary/80 text-xs uppercase" variant="secondary">
                                    {result.server}
                                </Badge>
                                <span className="text-muted-foreground text-xs">Lv. {result.level}</span>
                            </div>
                        </div>
                    </div>

                    {/* Additional info */}
                    <div className="mt-3 flex items-center justify-between border-border/50 border-t pt-3 text-muted-foreground text-xs">
                        <span>UID: {result.uid}</span>
                        <span className="font-mono">{(result.total_score ?? 0).toLocaleString()} pts</span>
                    </div>
                </CardContent>
            </Card>
        </Link>
    );

    return (
        <UserHoverCard result={result} side="top">
            {cardContent}
        </UserHoverCard>
    );
});
