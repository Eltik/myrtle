import { Clock, User } from "lucide-react";
import Image from "next/image";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import type { SearchResultEntry } from "~/types/api";
import { getAvatarURL } from "../../leaderboard/impl/constants";
import { GradeBadge } from "../../leaderboard/impl/grade-badge";
import { HOVER_DELAY } from "./constants";
import { formatRelativeTime, formatSecretaryName } from "./helpers";

interface UserHoverCardProps {
    result: SearchResultEntry;
    children: React.ReactNode;
    side?: "top" | "bottom" | "left" | "right";
}

export function UserHoverCard({ result, children, side = "top" }: UserHoverCardProps) {
    // Use only fields available in the base search response (no lazy loading needed)
    // This avoids fetching the massive 4MB+ data field for each user
    const secretary = result.secretary;
    const secretaryDisplay = formatSecretaryName(secretary);
    const secretaryAvatarURL = secretary ? getAvatarURL(secretary) : null;

    return (
        <HoverCard closeDelay={50} openDelay={HOVER_DELAY}>
            <HoverCardTrigger asChild>{children}</HoverCardTrigger>
            <HoverCardContent className="w-80 p-4" side={side}>
                <div className="flex items-start gap-4">
                    {/* Avatar */}
                    <Avatar className="h-16 w-16 shrink-0 border border-border">
                        <AvatarImage alt={result.nickname} src={getAvatarURL(result.avatarId) || "/placeholder.svg"} />
                        <AvatarFallback className="text-lg">{result.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                    </Avatar>

                    <div className="min-w-0 flex-1 space-y-1">
                        {/* Name and grade */}
                        <div className="flex items-center justify-between gap-2">
                            <h4 className="truncate font-semibold text-base">{result.nickname}</h4>
                            <GradeBadge grade={result.grade} size="sm" />
                        </div>

                        {/* Level and Server */}
                        <p className="text-muted-foreground text-sm">
                            Level {result.level} · <span className="uppercase">{result.server}</span>
                        </p>

                        {/* Score */}
                        <p className="font-mono text-muted-foreground text-xs">{result.totalScore.toLocaleString()} pts</p>
                    </div>
                </div>

                {/* Additional Info Section */}
                <div className="mt-4 space-y-2 border-t pt-3">
                    {/* Secretary/Assistant */}
                    {secretaryDisplay && (
                        <div className="flex items-center gap-2 text-sm">
                            {secretaryAvatarURL ? (
                                <div className="relative h-5 w-5 shrink-0 overflow-hidden rounded">
                                    <Image alt={secretaryDisplay} className="object-cover" fill sizes="20px" src={secretaryAvatarURL} />
                                </div>
                            ) : (
                                <User className="h-4 w-4 shrink-0 text-muted-foreground" />
                            )}
                            <span className="text-muted-foreground">Assistant:</span>
                            <span className="truncate font-medium">{secretaryDisplay}</span>
                        </div>
                    )}

                    {/* Last Updated */}
                    <div className="flex items-center gap-2 text-sm">
                        <Clock className="h-4 w-4 shrink-0 text-muted-foreground" />
                        <span className="text-muted-foreground">Updated:</span>
                        <span className="font-medium">{formatRelativeTime(result.updatedAt)}</span>
                    </div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}
