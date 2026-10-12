import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Resolution } from "#/types/generated/Resolution";
import { daysFromToday, formatDate, formatDateRange, relativeDays, useReleaseTimeZone } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { isOverdue } from "../resolution";
import type { messages } from "./ResolutionBadge.messages";

/** This badge renders its own chrome plus the date wording `helpers.ts` derives. */
type BadgeT = TypedT<typeof messages & typeof helperMessages>;

interface IResolutionBadgeProps {
    resolution: Resolution;
    today: Date;
    note?: string | null;
    caption?: React.ReactNode;
    /** A returning-players-only pool: its window is per player, so no calendar date is shown. */
    returning?: boolean;
    className?: string;
}

/** The rows with no EN date to show: a label, and a tooltip where the label needs one. */
function undated(status: "unmodelled" | "independent" | "unlisted", t: BadgeT): { label: string; hint?: string } {
    switch (status) {
        case "independent":
            return { label: t("release.badge.independent"), hint: t("release.badge.independent.title") };
        case "unlisted":
            return { label: t("release.badge.unlisted"), hint: t("release.badge.unlisted.title") };
        case "unmodelled":
            return { label: t("release.badge.noEstimate") };
    }
}

export function ResolutionBadge({ resolution, today, note, caption, returning, className }: IResolutionBadgeProps): React.ReactElement {
    const t: BadgeT = useT("tools");
    const locale = useLocale();
    const timeZone = useReleaseTimeZone();
    // Its table window is a placeholder years long (the game times it per player), so
    // neither the EN date nor the CN-derived status says anything a reader can plan on.
    if (returning) {
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                <Tooltip>
                    <TooltipTrigger render={<Badge variant="secondary" />}>{t("release.badge.returning")}</TooltipTrigger>
                    <TooltipPopup className="max-w-xs">{t("release.badge.returning.title")}</TooltipPopup>
                </Tooltip>
                {note && <span className="font-sans text-[11px] text-muted-foreground">{note}</span>}
            </div>
        );
    }
    if (resolution.status === "unmodelled" || resolution.status === "independent" || resolution.status === "unlisted") {
        const { label, hint } = undated(resolution.status, t);
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                {hint ? (
                    <Tooltip>
                        <TooltipTrigger render={<Badge variant="secondary" className="text-muted-foreground" />}>{label}</TooltipTrigger>
                        <TooltipPopup className="max-w-xs">{hint}</TooltipPopup>
                    </Tooltip>
                ) : (
                    <Badge variant="secondary" className="text-muted-foreground">
                        {label}
                    </Badge>
                )}
                {note && <span className="font-sans text-[11px] text-muted-foreground">{note}</span>}
            </div>
        );
    }

    let rel = relativeDays(daysFromToday(resolution.enStart, today), t);
    let chip: React.ReactNode;
    let dateText: string;
    let title: string | undefined;
    let sub: React.ReactNode = null;

    switch (resolution.status) {
        case "confirmed":
            chip = <Badge variant="success">{t("release.badge.confirmed")}</Badge>;
            dateText = formatDateRange(resolution.enStart, resolution.enEnd, locale, t, timeZone);
            break;
        case "override":
            title = resolution.note ? t("release.badge.overrideTitle", { source: resolution.source, note: resolution.note }) : resolution.source;
            // The whole block below is the tooltip trigger for `title`, so the chip carries none of its own.
            chip = <Badge variant="info">{t("release.badge.announced")}</Badge>;
            dateText = formatDateRange(resolution.enStart, resolution.enEnd, locale, t, timeZone);
            sub = <span className="truncate font-sans text-[11px] text-muted-foreground">{t("release.badge.per", { source: resolution.source })}</span>;
            break;
        case "estimated":
            if (isOverdue(resolution)) {
                title = t("release.badge.overdue.title");
                chip = <Badge variant="warning">{t("release.badge.overdue")}</Badge>;
                dateText = t("release.badge.overdue.was", { date: formatDate(resolution.estimatedStart, locale, timeZone) });
                rel = relativeDays(daysFromToday(resolution.estimatedStart, today), t);
                break;
            }
            chip = <Badge variant="outline">{t("release.badge.estimated")}</Badge>;
            dateText = formatDate(resolution.enStart, locale, timeZone);
            sub = <span className="font-mono text-[11px] text-muted-foreground">{t("release.badge.range", { lo: formatDate(resolution.lo, locale, timeZone), hi: formatDate(resolution.hi, locale, timeZone) })}</span>;
            break;
    }

    const blockClassName = cn("flex min-w-0 flex-col items-start gap-0.5 sm:items-end", className);
    const body = (
        <>
            <div className="flex flex-wrap items-center gap-1.5 sm:justify-end">
                {chip}
                <span className="font-medium font-sans text-[13px] text-foreground tabular-nums">{dateText}</span>
                <span className="font-sans text-[11px] text-muted-foreground">{rel}</span>
            </div>
            {sub}
            {caption && <span className="font-sans text-[11px] text-muted-foreground">{caption}</span>}
            {note && <span className="font-sans font-semibold text-[11px] text-warning-foreground">{note}</span>}
        </>
    );
    if (!title) return <div className={blockClassName}>{body}</div>;
    return (
        <Tooltip>
            <TooltipTrigger render={<div className={blockClassName} />}>{body}</TooltipTrigger>
            <TooltipPopup className="max-w-xs">{title}</TooltipPopup>
        </Tooltip>
    );
}
