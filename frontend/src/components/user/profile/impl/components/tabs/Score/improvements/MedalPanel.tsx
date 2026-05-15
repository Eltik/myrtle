import { useMemo, useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
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

export function MedalPanel({ improvements, accent }: IProps) {
    const { permanent_missing: permanent, event_in_window_missing: event } = improvements.medals;
    if (permanent.length === 0 && event.length === 0) {
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
        </div>
    );
}

function MedalList({ title, subtitle, medals, accent, mode }: { title: string; subtitle: string; medals: IMedalGap[]; accent: string; mode: "permanent" | "event" }) {
    const [showAll, setShowAll] = useState(false);
    const visible = useMemo(() => (showAll ? medals : medals.slice(0, INITIAL_VISIBLE)), [showAll, medals]);

    return (
        <div className="flex flex-col gap-2">
            <SectionHeader title={title} count={`${medals.length} missing`} accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>{subtitle}</p>
            <div className="flex flex-col gap-1.5">
                {visible.map((m) => (
                    <MedalRow key={m.medal_id} medal={m} mode={mode} />
                ))}
            </div>
            {medals.length > INITIAL_VISIBLE && <ShowMoreButton onClick={() => setShowAll((s) => !s)} label={showAll ? "Show less" : `Show ${medals.length - INITIAL_VISIBLE} more`} />}
        </div>
    );
}

function MedalRow({ medal, mode }: { medal: IMedalGap; mode: "permanent" | "event" }) {
    const color = medalRarityColor(medal.rarity);
    const daysLeft = medal.end_time ? Math.max(0, Math.ceil((medal.end_time * 1000 - Date.now()) / 86_400_000)) : null;
    return (
        <div className="flex items-start gap-2.5 rounded-md border border-border/40 bg-muted/15 px-3 py-2 transition-colors hover:border-border/65 hover:bg-muted/25">
            <Pill color={color} className="mt-0.5 shrink-0">
                {rarityLabel(medal.rarity)}
            </Pill>
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
                <Pill color={daysLeft <= 7 ? URGENT_COLOR : color} className={cn("mt-0.5 shrink-0 whitespace-nowrap", TEXT_BADGE)}>
                    {daysLeft <= 0 ? "ending now" : `${daysLeft}d left`}
                </Pill>
            )}
        </div>
    );
}
