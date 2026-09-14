import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { cn } from "#/lib/utils";
import type { Resolution } from "#/types/generated/Resolution";
import { daysFromToday, formatDate, formatDateRange, relativeDays } from "../helpers";

interface IResolutionBadgeProps {
    resolution: Resolution;
    today: Date;
    note?: string | null;
    caption?: React.ReactNode;
    standing?: boolean;
    className?: string;
}

export function ResolutionBadge({ resolution, today, note, caption, standing, className }: IResolutionBadgeProps): React.ReactElement {
    if (resolution.status === "unmodelled" || resolution.status === "independent") {
        const independent = resolution.status === "independent";
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                <Badge variant="secondary" className="text-muted-foreground" title={independent ? "EN schedules this banner kind on its own calendar; the CN date says nothing about the EN date." : undefined}>
                    {independent ? "EN schedules separately" : "No estimate"}
                </Badge>
                {note && <span className="font-sans text-[11px] text-muted-foreground">{note}</span>}
            </div>
        );
    }

    const rel = relativeDays(daysFromToday(resolution.enStart, today));
    let chip: React.ReactNode;
    let dateText: string;
    let title: string | undefined;
    let sub: React.ReactNode = null;

    switch (resolution.status) {
        case "confirmed":
            chip = <Badge variant="default">Confirmed</Badge>;
            dateText = formatDateRange(resolution.enStart, resolution.enEnd);
            break;
        case "override":
            title = resolution.note ? `${resolution.source}: ${resolution.note}` : resolution.source;
            chip = (
                <Badge variant="info" title={title}>
                    Announced
                </Badge>
            );
            dateText = formatDateRange(resolution.enStart, resolution.enEnd);
            sub = <span className="truncate font-sans text-[11px] text-muted-foreground">per {resolution.source}</span>;
            break;
        case "estimated":
            chip = <Badge variant="outline">Estimated</Badge>;
            dateText = formatDate(resolution.enStart);
            sub = (
                <span className="font-mono text-[11px] text-muted-foreground">
                    {formatDate(resolution.lo)} to {formatDate(resolution.hi)}
                </span>
            );
            break;
    }

    return (
        <div className={cn("flex min-w-0 flex-col items-start gap-0.5 sm:items-end", className)} title={title}>
            <div className="flex flex-wrap items-center gap-1.5 sm:justify-end">
                {chip}
                <span className="font-medium font-sans text-[13px] text-foreground tabular-nums">{standing ? `since ${formatDate(resolution.enStart)}` : dateText}</span>
                <span className="font-sans text-[11px] text-muted-foreground">{standing ? "standing pool" : rel}</span>
            </div>
            {sub}
            {caption && <span className="font-sans text-[11px] text-muted-foreground">{caption}</span>}
            {note && <span className="font-sans font-semibold text-[11px] text-warning-foreground">{note}</span>}
        </div>
    );
}
