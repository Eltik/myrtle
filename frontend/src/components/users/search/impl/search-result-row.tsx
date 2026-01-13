import { Calendar, Users } from "lucide-react";
import Link from "next/link";
import React from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { SearchResultEntry } from "~/types/api";
import { getAvatarURL } from "../../leaderboard/impl/constants";
import { formatAccountAge, formatRegistrationDate, getOperatorCount, getStatusData } from "./helpers";
import { UserHoverCard } from "./user-hover-card";

interface SearchResultRowProps {
    result: SearchResultEntry;
}

export const SearchResultRow = React.memo(function SearchResultRow({ result }: SearchResultRowProps) {
    const status = getStatusData(result);
    const registerTs = status?.registerTs as number | undefined;
    const accountAge = formatAccountAge(registerTs);
    const registrationDate = formatRegistrationDate(registerTs);
    const operatorCount = getOperatorCount(result);

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
                        <Badge className="shrink-0 border border-border bg-secondary/80 text-xs uppercase" variant="secondary">
                            {result.server}
                        </Badge>
                    </div>
                    <p className="text-muted-foreground text-sm">
                        Level {result.level} · UID: {result.uid}
                    </p>
                </div>

                <div className="hidden items-center gap-4 sm:flex">
                    {/* Account age */}
                    {accountAge && (
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <div className="text-right">
                                    <p className="flex items-center justify-end gap-1 text-muted-foreground text-xs">
                                        <Calendar className="h-3 w-3" />
                                        Account
                                    </p>
                                    <p className="text-sm">{accountAge}</p>
                                </div>
                            </TooltipTrigger>
                            <TooltipContent variant="dark">Registered: {registrationDate}</TooltipContent>
                        </Tooltip>
                    )}
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

                {/* Mobile: show operator count */}
                <div className="flex items-center sm:hidden">
                    {operatorCount && (
                        <div className="flex shrink-0 items-center gap-1 rounded border border-border bg-muted/50 px-1.5 py-0.5">
                            <Users className="h-3 w-3 text-muted-foreground" />
                            <span className="font-medium text-xs">{operatorCount}</span>
                        </div>
                    )}
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
