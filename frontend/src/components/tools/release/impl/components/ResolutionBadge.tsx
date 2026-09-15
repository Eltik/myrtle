import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Resolution } from "#/types/generated/Resolution";
import { daysFromToday, formatDate, formatDateRange, relativeDays } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import type { messages } from "./ResolutionBadge.messages";

/** This badge renders its own chrome plus the date wording `helpers.ts` derives. */
type BadgeT = TypedT<typeof messages & typeof helperMessages>;

interface IResolutionBadgeProps {
    resolution: Resolution;
    today: Date;
    note?: string | null;
    caption?: React.ReactNode;
    standing?: boolean;
    className?: string;
}

export function ResolutionBadge({ resolution, today, note, caption, standing, className }: IResolutionBadgeProps): React.ReactElement {
    const t: BadgeT = useT("tools");
    const locale = useLocale();
    if (resolution.status === "unmodelled" || resolution.status === "independent") {
        const independent = resolution.status === "independent";
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                <Badge variant="secondary" className="text-muted-foreground" title={independent ? t("release.badge.independent.title") : undefined}>
                    {independent ? t("release.badge.independent") : t("release.badge.noEstimate")}
                </Badge>
                {note && <span className="font-sans text-[11px] text-muted-foreground">{note}</span>}
            </div>
        );
    }

    const rel = relativeDays(daysFromToday(resolution.enStart, today), t);
    let chip: React.ReactNode;
    let dateText: string;
    let title: string | undefined;
    let sub: React.ReactNode = null;

    switch (resolution.status) {
        case "confirmed":
            chip = <Badge variant="success">{t("release.badge.confirmed")}</Badge>;
            dateText = formatDateRange(resolution.enStart, resolution.enEnd, locale, t);
            break;
        case "override":
            title = resolution.note ? t("release.badge.overrideTitle", { source: resolution.source, note: resolution.note }) : resolution.source;
            chip = (
                <Badge variant="info" title={title}>
                    {t("release.badge.announced")}
                </Badge>
            );
            dateText = formatDateRange(resolution.enStart, resolution.enEnd, locale, t);
            sub = <span className="truncate font-sans text-[11px] text-muted-foreground">{t("release.badge.per", { source: resolution.source })}</span>;
            break;
        case "estimated":
            chip = <Badge variant="outline">{t("release.badge.estimated")}</Badge>;
            dateText = formatDate(resolution.enStart, locale);
            sub = <span className="font-mono text-[11px] text-muted-foreground">{t("release.badge.range", { lo: formatDate(resolution.lo, locale), hi: formatDate(resolution.hi, locale) })}</span>;
            break;
    }

    return (
        <div className={cn("flex min-w-0 flex-col items-start gap-0.5 sm:items-end", className)} title={title}>
            <div className="flex flex-wrap items-center gap-1.5 sm:justify-end">
                {chip}
                <span className="font-medium font-sans text-[13px] text-foreground tabular-nums">{standing ? t("release.badge.since", { date: formatDate(resolution.enStart, locale) }) : dateText}</span>
                <span className="font-sans text-[11px] text-muted-foreground">{standing ? t("release.badge.standing") : rel}</span>
            </div>
            {sub}
            {caption && <span className="font-sans text-[11px] text-muted-foreground">{caption}</span>}
            {note && <span className="font-sans font-semibold text-[11px] text-warning-foreground">{note}</span>}
        </div>
    );
}
