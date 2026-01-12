import { Calendar, Users } from "lucide-react";
import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { SearchResultEntry } from "~/types/api";
import { getAvatarUrl } from "../../leaderboard/impl/constants";
import { formatAccountAge, formatRegistrationDate, getOperatorCount, getStatusData } from "./helpers";
import { UserHoverCard } from "./user-hover-card";

interface SearchResultCardProps {
    result: SearchResultEntry;
}

export const SearchResultCard = React.memo(function SearchResultCard({ result }: SearchResultCardProps) {
    const status = getStatusData(result);
    const registerTs = status?.registerTs as number | undefined;
    const accountAge = formatAccountAge(registerTs);
    const registrationDate = formatRegistrationDate(registerTs);
    const operatorCount = getOperatorCount(result);

    const cardContent = (
        <Link href={`/user/${result.uid}`}>
            <Card className="group h-full overflow-hidden py-0 transition-all duration-200 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/5">
                <CardContent className="p-4">
                    <div className="flex items-start gap-3">
                        <Avatar className="h-14 w-14 shrink-0 border border-border transition-transform duration-200 group-hover:scale-105">
                            <AvatarImage alt={result.nickname} src={getAvatarUrl(result.avatarId) || "/placeholder.svg"} />
                            <AvatarFallback className="text-sm">{result.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                        </Avatar>
                        <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                                <h3 className="truncate font-medium text-base transition-colors group-hover:text-primary">{result.nickname}</h3>
                            </div>
                            <div className="mt-1 flex flex-wrap items-center gap-1.5">
                                <Badge className="border border-border bg-secondary/80 text-xs uppercase" variant="secondary">
                                    {result.server}
                                </Badge>
                                <span className="text-muted-foreground text-xs">Lv. {result.level}</span>
                            </div>
                        </div>
                        {/* Operator count */}
                        {operatorCount && (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <div className="flex shrink-0 items-center gap-1.5 rounded-md border border-border bg-muted/50 px-2 py-1">
                                        <Users className="h-3.5 w-3.5 text-muted-foreground" />
                                        <span className="font-medium text-xs">{operatorCount}</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent variant="dark">{operatorCount} operators</TooltipContent>
                            </Tooltip>
                        )}
                    </div>

                    {/* Additional info */}
                    <div className="mt-3 flex items-center justify-between border-border/50 border-t pt-3 text-muted-foreground text-xs">
                        <span>UID: {result.uid}</span>
                        {accountAge && (
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <span className="flex cursor-help items-center gap-1">
                                        <Calendar className="h-3 w-3" />
                                        {accountAge}
                                    </span>
                                </TooltipTrigger>
                                <TooltipContent variant="dark">Registered: {registrationDate}</TooltipContent>
                            </Tooltip>
                        )}
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
