import { Users } from "lucide-react";
import { useMemo, useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { env } from "#/env";
import type { IImprovementsResponse, IMedalGap } from "#/lib/api/user";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./MedalPanel.messages";
import { EmptyHint, medalRarityColor, PANEL_PADDING, Pill, SectionHeader, ShowMoreButton, TEXT_BADGE, TEXT_BODY, TEXT_KICKER, TEXT_META, URGENT_COLOR } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";

/** The panel also renders the show-more chrome declared in `shared.messages.ts`. */
type PanelT = TypedT<typeof messages & typeof sharedMessages>;

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

function rarityLabel(rarity: string): string {
    return rarity.replace("D5", ".5");
}

function ownedPctLabel(pct: number): string {
    if (pct > 0 && pct < 1) return "<1%";
    return `${Math.round(pct)}%`;
}

function medalIconURL(medalId: string): string {
    return `${env.VITE_BACKEND_URL ?? ""}/api/medal-icon/${medalId}`;
}

function MedalIcon({ medal, color }: { medal: IMedalGap; color: string }) {
    const [errored, setErrored] = useState(false);
    return (
        <span
            className="relative grid size-9 shrink-0 place-items-center rounded-md border bg-muted/15"
            style={{
                borderColor: `color-mix(in oklch, ${color} 35%, var(--border))`,
                background: `radial-gradient(ellipse at 30% 25%, color-mix(in oklch, ${color} 18%, transparent), transparent 70%), color-mix(in oklch, var(--muted) 35%, transparent)`,
            }}
            aria-hidden
        >
            {!errored ? (
                <img alt="" src={medalIconURL(medal.medal_id)} width={32} height={32} className="h-8 w-8 object-contain" decoding="async" loading="lazy" onError={() => setErrored(true)} />
            ) : (
                <span className={cn(TEXT_BADGE, "font-semibold")} style={{ color: `color-mix(in oklch, ${color} 75%, var(--foreground))` }}>
                    {rarityLabel(medal.rarity)}
                </span>
            )}
        </span>
    );
}

type MedalMode = "permanent" | "event" | "locked" | "unobtainable";

export function MedalPanel({ improvements, accent }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const { permanent_missing: permanent, event_in_window_missing: event, operator_locked: locked, unobtainable_missing: unobtainable } = improvements.medals;
    if (permanent.length === 0 && event.length === 0 && locked.length === 0 && unobtainable.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>{t("score.improvements.medal.empty")}</EmptyHint>
            </div>
        );
    }

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {event.length > 0 && <MedalList title={t("score.improvements.medal.event.title")} subtitle={t("score.improvements.medal.event.subtitle")} medals={event} accent={accent} mode="event" />}
            {permanent.length > 0 && <MedalList title={t("score.improvements.medal.permanent.title")} subtitle={t("score.improvements.medal.permanent.subtitle")} medals={permanent} accent={accent} mode="permanent" />}
            {locked.length > 0 && <MedalList title={t("score.improvements.medal.locked.title")} subtitle={t("score.improvements.medal.locked.subtitle")} medals={locked} accent={accent} mode="locked" />}
            {unobtainable.length > 0 && <MedalList title={t("score.improvements.medal.unobtainable.title")} subtitle={t("score.improvements.medal.unobtainable.subtitle")} medals={unobtainable} accent={accent} mode="unobtainable" startCollapsed />}
        </div>
    );
}

function MedalList({ title, subtitle, medals, accent, mode, startCollapsed = false }: { title: string; subtitle: string; medals: IMedalGap[]; accent: string; mode: MedalMode; startCollapsed?: boolean }) {
    const t: PanelT = useT("user");
    const [collapsed, setCollapsed] = useState(startCollapsed);
    const [showAll, setShowAll] = useState(false);
    const visible = useMemo(() => (showAll ? medals : medals.slice(0, INITIAL_VISIBLE)), [showAll, medals]);

    return (
        <div className="flex flex-col gap-2">
            <SectionHeader title={title} count={t("score.improvements.medal.missingCount", { n: medals.length })} accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>{subtitle}</p>
            {collapsed ? (
                <ShowMoreButton onClick={() => setCollapsed(false)} label={t("score.improvements.medal.show", { n: medals.length })} />
            ) : (
                <>
                    <div className="flex flex-col gap-1.5">
                        {visible.map((m) => (
                            <MedalRow key={m.medal_id} medal={m} mode={mode} />
                        ))}
                    </div>
                    {medals.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll((s) => !s)} label={showAll ? t("score.improvements.showLess") : t("score.improvements.showMore", { n: medals.length - INITIAL_VISIBLE })} />}
                    {startCollapsed && (
                        <ShowMoreButton
                            onClick={() => {
                                setCollapsed(true);
                                setShowAll(false);
                            }}
                            label={t("score.improvements.medal.hide")}
                        />
                    )}
                </>
            )}
        </div>
    );
}

function MedalRow({ medal, mode }: { medal: IMedalGap; mode: MedalMode }) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const color = medalRarityColor(medal.rarity);
    const daysLeft = medal.end_time ? Math.max(0, Math.ceil((medal.end_time * 1000 - Date.now()) / 86_400_000)) : null;
    const endedLabel = medal.end_time ? f.date(new Date(medal.end_time * 1000), { year: "numeric", month: "short", day: "numeric" }) : null;
    const lock = medal.operator_lock;
    return (
        <div className={cn("flex items-center gap-2.5 rounded-md border border-border/40 bg-muted/15 px-3 py-2 transition-colors hover:border-border/65 hover:bg-muted/25", (mode === "locked" || mode === "unobtainable") && "opacity-70")}>
            <Tooltip>
                <TooltipTrigger
                    render={
                        <span className="shrink-0">
                            <MedalIcon medal={medal} color={color} />
                        </span>
                    }
                />
                <TooltipContent sideOffset={4}>
                    <p>{rarityLabel(medal.rarity)}</p>
                </TooltipContent>
            </Tooltip>
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className={cn(TEXT_BODY, "truncate font-medium")}>{medal.name}</span>
                    {medal.is_hidden && (
                        <Tooltip>
                            <TooltipTrigger render={<span className={cn(TEXT_KICKER, "rounded border border-border/40 px-1 py-0.5 text-muted-foreground")}>{t("score.improvements.medal.hidden")}</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>{t("score.improvements.medal.hidden.tooltip")}</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
                {medal.get_method && <p className={cn(TEXT_META, "mt-0.5 text-muted-foreground/85")}>{medal.get_method}</p>}
            </div>
            {typeof medal.owned_pct === "number" && (
                <Tooltip>
                    <TooltipTrigger
                        render={
                            <span className={cn(TEXT_META, "flex shrink-0 items-center gap-1 whitespace-nowrap text-muted-foreground tabular-nums")}>
                                <Users className="size-3" aria-hidden />
                                {ownedPctLabel(medal.owned_pct)}
                            </span>
                        }
                    />
                    <TooltipContent sideOffset={4}>
                        <p>{t("score.improvements.medal.ownedTooltip", { pct: medal.owned_pct })}</p>
                    </TooltipContent>
                </Tooltip>
            )}
            {mode === "event" && daysLeft !== null && (
                <Pill color={daysLeft <= 7 ? URGENT_COLOR : color} className={cn("shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                    {daysLeft <= 0 ? t("score.improvements.medal.endingNow") : t("score.improvements.medal.daysLeft", { days: daysLeft })}
                </Pill>
            )}
            {mode === "unobtainable" && endedLabel && (
                <Pill color={color} className={cn("shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                    {t("score.improvements.medal.ended", { date: endedLabel })}
                </Pill>
            )}
            {mode === "locked" && lock && (
                <Tooltip>
                    <TooltipTrigger
                        render={
                            <Pill color={color} className={cn("shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                                {lock.reason}
                            </Pill>
                        }
                    />
                    <TooltipContent sideOffset={4}>
                        <p>{t("score.improvements.medal.lockTooltip", { operator: lock.operatorName, reason: lock.reason })}</p>
                    </TooltipContent>
                </Tooltip>
            )}
        </div>
    );
}
