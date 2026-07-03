import { useMemo, useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { env } from "#/env";
import type { IImprovementsResponse, IMedalGap } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, medalRarityColor, PANEL_PADDING, Pill, SectionHeader, ShowMoreButton, TEXT_BADGE, TEXT_BODY, TEXT_KICKER, TEXT_META, URGENT_COLOR } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

function rarityLabel(rarity: string): string {
    return rarity.replace("D5", ".5");
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
    const { permanent_missing: permanent, event_in_window_missing: event, operator_locked: locked, unobtainable_missing: unobtainable } = improvements.medals;
    if (permanent.length === 0 && event.length === 0 && locked.length === 0 && unobtainable.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>You've earned every medal that's currently reachable. Nice work.</EmptyHint>
            </div>
        );
    }

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {event.length > 0 && <MedalList title="Event medals - limited time" subtitle="Sorted by earliest ending. Hidden medals are still listed but their hint is suppressed." medals={event} accent={accent} mode="event" />}
            {permanent.length > 0 && <MedalList title="Permanent medals" subtitle="Sorted by rarity desc. Highest-rarity gaps first." medals={permanent} accent={accent} mode="permanent" />}
            {locked.length > 0 && (
                <MedalList
                    title="Behind unavailable operators"
                    subtitle="These require a collab operator you don't own. Since collab operators aren't normally obtainable, these medals don't count toward your medal score. Earn the operator in a rerun and they'll count again. Shown for reference."
                    medals={locked}
                    accent={accent}
                    mode="locked"
                />
            )}
            {unobtainable.length > 0 && (
                <MedalList
                    title="No longer obtainable"
                    subtitle="These medals' windows have passed and won't reopen. Past event medals still count in your score with decaying weight rather than against the permanent pool; one-time modes and retired towers are excluded. Shown for reference."
                    medals={unobtainable}
                    accent={accent}
                    mode="unobtainable"
                    startCollapsed
                />
            )}
        </div>
    );
}

function MedalList({ title, subtitle, medals, accent, mode, startCollapsed = false }: { title: string; subtitle: string; medals: IMedalGap[]; accent: string; mode: MedalMode; startCollapsed?: boolean }) {
    const [collapsed, setCollapsed] = useState(startCollapsed);
    const [showAll, setShowAll] = useState(false);
    const visible = useMemo(() => (showAll ? medals : medals.slice(0, INITIAL_VISIBLE)), [showAll, medals]);

    return (
        <div className="flex flex-col gap-2">
            <SectionHeader title={title} count={`${medals.length} missing`} accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>{subtitle}</p>
            {collapsed ? (
                <ShowMoreButton onClick={() => setCollapsed(false)} label={`Show ${medals.length}`} />
            ) : (
                <>
                    <div className="flex flex-col gap-1.5">
                        {visible.map((m) => (
                            <MedalRow key={m.medal_id} medal={m} mode={mode} />
                        ))}
                    </div>
                    {medals.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll((s) => !s)} label={showAll ? "Show less" : `Show ${medals.length - INITIAL_VISIBLE} more`} />}
                    {startCollapsed && (
                        <ShowMoreButton
                            onClick={() => {
                                setCollapsed(true);
                                setShowAll(false);
                            }}
                            label="Hide"
                        />
                    )}
                </>
            )}
        </div>
    );
}

function MedalRow({ medal, mode }: { medal: IMedalGap; mode: MedalMode }) {
    const color = medalRarityColor(medal.rarity);
    const daysLeft = medal.end_time ? Math.max(0, Math.ceil((medal.end_time * 1000 - Date.now()) / 86_400_000)) : null;
    const endedLabel = medal.end_time ? new Date(medal.end_time * 1000).toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" }) : null;
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
                            <TooltipTrigger render={<span className={cn(TEXT_KICKER, "rounded border border-border/40 px-1 py-0.5 text-muted-foreground")}>Hidden</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>Hidden medal - unlock condition isn't shown in-game.</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
                {medal.get_method && <p className={cn(TEXT_META, "mt-0.5 text-muted-foreground/85")}>{medal.get_method}</p>}
            </div>
            {mode === "event" && daysLeft !== null && (
                <Pill color={daysLeft <= 7 ? URGENT_COLOR : color} className={cn("shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                    {daysLeft <= 0 ? "ending now" : `${daysLeft}d left`}
                </Pill>
            )}
            {mode === "unobtainable" && endedLabel && (
                <Pill color={color} className={cn("shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                    Ended {endedLabel}
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
                        <p>
                            Requires {lock.operatorName}, a {lock.reason} operator you don't own. Excluded from your medal score until you have them.
                        </p>
                    </TooltipContent>
                </Tooltip>
            )}
        </div>
    );
}
