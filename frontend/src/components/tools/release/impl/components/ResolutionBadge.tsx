import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Resolution } from "#/types/generated/Resolution";
import { daysFromToday, formatDate, formatDateRange, relativeDays } from "../helpers";
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
    // Its table window is a placeholder years long (the game times it per player), so
    // neither the EN date nor the CN-derived status says anything a reader can plan on.
    if (returning) {
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                <Badge variant="secondary" title={t("release.badge.returning.title")}>
                    {t("release.badge.returning")}
                </Badge>
                {note && <span className="font-sans text-[11px] text-muted-foreground">{note}</span>}
            </div>
        );
    }
    if (resolution.status === "unmodelled" || resolution.status === "independent" || resolution.status === "unlisted") {
        const { label, hint } = undated(resolution.status, t);
        return (
            <div className={cn("flex flex-col items-start gap-0.5 sm:items-end", className)}>
                <Badge variant="secondary" className="text-muted-foreground" title={hint}>
                    {label}
                </Badge>
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
            if (isOverdue(resolution)) {
                title = t("release.badge.overdue.title");
                chip = <Badge variant="warning">{t("release.badge.overdue")}</Badge>;
                dateText = t("release.badge.overdue.was", { date: formatDate(resolution.estimatedStart, locale) });
                rel = relativeDays(daysFromToday(resolution.estimatedStart, today), t);
                break;
            }
            chip = <Badge variant="outline">{t("release.badge.estimated")}</Badge>;
            dateText = formatDate(resolution.enStart, locale);
            sub = <span className="font-mono text-[11px] text-muted-foreground">{t("release.badge.range", { lo: formatDate(resolution.lo, locale), hi: formatDate(resolution.hi, locale) })}</span>;
            break;
    }

    return (
        <div className={cn("flex min-w-0 flex-col items-start gap-0.5 sm:items-end", className)} title={title}>
            <div className="flex flex-wrap items-center gap-1.5 sm:justify-end">
                {chip}
                <span className="font-medium font-sans text-[13px] text-foreground tabular-nums">{dateText}</span>
                <span className="font-sans text-[11px] text-muted-foreground">{rel}</span>
            </div>
            {sub}
            {caption && <span className="font-sans text-[11px] text-muted-foreground">{caption}</span>}
            {note && <span className="font-sans font-semibold text-[11px] text-warning-foreground">{note}</span>}
        </div>
    );
}
