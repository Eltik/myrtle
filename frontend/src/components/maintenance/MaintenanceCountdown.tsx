import { WrenchIcon } from "lucide-react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { type IMaintenanceState, useMaintenance } from "#/hooks/use-maintenance";
import { useTheme } from "#/hooks/use-theme";
import { useT } from "#/lib/i18n";
import { useFormatters } from "#/lib/i18n/formatters";
import type { TypedT } from "#/lib/i18n/messages";
import { formatSpan, isGameDown } from "#/lib/maintenance";
import { cn } from "#/lib/utils";
import type { messages } from "./Maintenance.messages";

type T = TypedT<typeof messages>;

function shortLabel(m: IMaintenanceState, t: T): string {
    const time = m.secondsLeft === null ? "" : formatSpan(m.secondsLeft);
    switch (m.stage) {
        case "upcoming":
            return t("maintenance.short.upcoming", { time });
        case "ongoing":
            return t("maintenance.short.ongoing", { time });
        case "extended":
            return t("maintenance.short.extended", { time });
        case "overrun":
            return t("maintenance.short.overrun");
        case "openEnded":
            return t("maintenance.short.openEnded");
    }
}

/** A dot that says "down right now" without shouting: amber while down, muted before. */
function StatusDot({ down, className }: { down: boolean; className?: string }) {
    return <span aria-hidden="true" className={cn("size-1.5 shrink-0 rounded-full", down ? "bg-amber-500" : "bg-muted-foreground/60", className)} />;
}

/**
 * The top-bar pill. Links to Yostar's own notice. Hidden when Settings > Top Bar turns it off.
 * Below `md` it is a 32px icon square with the dot pinned to its corner: the bar has no room
 * to spare on a phone.
 */
export function MaintenancePill() {
    const m = useMaintenance();
    const { maintenanceCountdown } = useTheme();
    const t: T = useT("common");
    if (!m || !maintenanceCountdown) return null;
    const label = shortLabel(m, t);
    return (
        <Tooltip>
            <TooltipTrigger
                render={
                    // biome-ignore lint/a11y/useAnchorContent: the content arrives as TooltipTrigger children, and the aria-label names it
                    <a
                        href={m.window.notice_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        aria-label={`${label}. ${t("maintenance.pillLabel")}`}
                        className="relative inline-flex size-8 shrink-0 items-center justify-center gap-1.5 rounded-md border border-border bg-muted/60 font-medium text-[12px] text-foreground tabular-nums no-underline transition-colors pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:-inset-y-1.5 hover:bg-muted md:w-auto md:px-2"
                    />
                }
            >
                <WrenchIcon className="size-3.5 text-muted-foreground" aria-hidden="true" />
                <StatusDot down={isGameDown(m.stage)} className="max-md:absolute max-md:top-1.5 max-md:right-1.5" />
                <span className="hidden whitespace-nowrap md:inline">{label}</span>
            </TooltipTrigger>
            {/* The pill shows the label itself from md up; below that it is icon-only. */}
            <TooltipPopup className="md:hidden">{label}</TooltipPopup>
        </Tooltip>
    );
}

/** The same window on Home, with the times spelled out. Shows whatever the top-bar setting says. */
export function MaintenanceBanner({ className }: { className?: string }) {
    const m = useMaintenance();
    const t: T = useT("common");
    const { date } = useFormatters();
    if (!m) return null;
    const at = (sec: number) => date(sec * 1000, { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" });
    const start = at(m.window.starts_at);
    const end = m.window.ends_at === null ? null : at(m.window.ends_at);
    let detail: string;
    switch (m.stage) {
        case "upcoming":
            detail = end ? t("maintenance.detail.upcoming", { start, end }) : t("maintenance.detail.upcomingNoEnd", { start });
            break;
        case "ongoing":
            detail = t("maintenance.detail.ongoing", { start, end: end ?? "" });
            break;
        case "extended":
            detail = t("maintenance.detail.extended", { end: end ?? "" });
            break;
        case "overrun":
            detail = t("maintenance.detail.overrun", { end: end ?? "" });
            break;
        case "openEnded":
            detail = t("maintenance.detail.openEnded", { start });
            break;
    }
    return (
        <div className={cn("flex flex-wrap items-center gap-x-3 gap-y-1 rounded-[14px] border border-border bg-card px-4 py-3", className)}>
            <WrenchIcon className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
            <StatusDot down={isGameDown(m.stage)} />
            <span className="font-semibold text-[14px] text-foreground tabular-nums">{shortLabel(m, t)}</span>
            <span className="text-[13px] text-muted-foreground">{detail}</span>
            <a href={m.window.notice_url} target="_blank" rel="noopener noreferrer" className="pointer-coarse:-my-3 ml-auto pointer-coarse:py-3 text-[13px] text-primary underline-offset-4 hover:underline">
                {t("maintenance.notice")}
            </a>
        </div>
    );
}
