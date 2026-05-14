import { useMemo, useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IImprovementsResponse, IMedalGap } from "#/lib/api/user";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const INITIAL_VISIBLE = 12;

function rarityColor(rarity: string): string {
    if (rarity === "T3D5") return "oklch(0.78 0.18 30)"; // rare amber
    if (rarity === "T3") return "oklch(0.74 0.17 75)"; // gold
    if (rarity === "T2D5") return "oklch(0.70 0.15 75)";
    if (rarity === "T2") return "oklch(0.67 0.18 295)"; // purple
    if (rarity === "T1D5") return "oklch(0.60 0.13 220)";
    return "oklch(0.55 0.10 220)";
}

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
            {event.length > 0 && <MedalList title="Event medals — limited time" subtitle="Sorted by earliest ending. Hidden medals are still listed but their hint is suppressed." medals={event} accent={accent} mode="event" />}
            {permanent.length > 0 && <MedalList title="Permanent medals" subtitle="Sorted by rarity desc. Highest-rarity gaps first." medals={permanent} accent={accent} mode="permanent" />}
        </div>
    );
}

function MedalList({ title, subtitle, medals, accent, mode }: { title: string; subtitle: string; medals: IMedalGap[]; accent: string; mode: "permanent" | "event" }) {
    const [showAll, setShowAll] = useState(false);
    const visible = useMemo(() => (showAll ? medals : medals.slice(0, INITIAL_VISIBLE)), [showAll, medals]);

    return (
        <div className="flex flex-col gap-2.5">
            <SectionHeader title={title} count={`${medals.length} missing`} accent={accent} />
            <p className="text-[11px] text-muted-foreground">{subtitle}</p>
            <div className="flex flex-col gap-1.5">
                {visible.map((m) => (
                    <MedalRow key={m.medal_id} medal={m} accent={accent} mode={mode} />
                ))}
            </div>
            {medals.length > INITIAL_VISIBLE && (
                <button type="button" onClick={() => setShowAll(!showAll)} className="self-start font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground transition-colors hover:text-foreground">
                    {showAll ? "Show less" : `Show ${medals.length - INITIAL_VISIBLE} more`}
                </button>
            )}
        </div>
    );
}

function MedalRow({ medal, mode }: { medal: IMedalGap; accent: string; mode: "permanent" | "event" }) {
    const color = rarityColor(medal.rarity);
    const daysLeft = medal.end_time ? Math.max(0, Math.ceil((medal.end_time * 1000 - Date.now()) / 86_400_000)) : null;
    return (
        <div className="flex items-start gap-2.5 rounded-lg border border-border/40 bg-muted/15 px-3 py-2 transition-colors hover:border-border/65 hover:bg-muted/25">
            <Pill color={color} className="mt-0.5 shrink-0">
                {rarityLabel(medal.rarity)}
            </Pill>
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className="truncate text-[12.5px] font-medium leading-tight">{medal.name}</span>
                    {medal.is_hidden && (
                        <Tooltip>
                            <TooltipTrigger render={<span className="rounded border border-border/40 px-1 py-0.5 font-mono text-[8.5px] uppercase tracking-wider text-muted-foreground">Hidden</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>Hidden medal — unlock condition isn't shown in-game.</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
                {medal.get_method && <p className="mt-0.5 text-[11px] leading-snug text-muted-foreground/85">{medal.get_method}</p>}
            </div>
            {mode === "event" && daysLeft !== null && (
                <Pill color={daysLeft <= 7 ? "oklch(0.62 0.22 25)" : color} className="mt-0.5 shrink-0 whitespace-nowrap">
                    {daysLeft <= 0 ? "ending now" : `${daysLeft}d left`}
                </Pill>
            )}
        </div>
    );
}
