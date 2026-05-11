import { Dices, RotateCcw, Settings2 } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { StatTile } from "./StatTile";

interface IBriefingHeroProps {
    operatorsAvailable: number;
    operatorsRoster: number;
    stagesAvailable: number;
    hasResult: boolean;
    canRoll: boolean;
    onRollAll: () => void;
    onReset: () => void;
    onOpenSettings: () => void;
}

/**
 * Hero strip — "Operations Briefing" headline + big roll CTA on the left,
 * stat tiles on the right. Distinctive vs old: serif display heading,
 * monospaced uppercase kicker, hairline guilloche divider beneath the title.
 */
export function BriefingHero({ operatorsAvailable, operatorsRoster, stagesAvailable, hasResult, canRoll, onRollAll, onReset, onOpenSettings }: IBriefingHeroProps): React.ReactElement {
    return (
        <section className="relative overflow-hidden rounded-2xl border border-border/70 bg-card text-card-foreground shadow-xs/5">
            <Guilloche />
            <div className="relative grid gap-6 p-5 sm:p-7 lg:grid-cols-[1fr_auto] lg:gap-10 lg:p-8">
                <div className="min-w-0">
                    <p className="font-mono text-[11px] uppercase tracking-[0.32em] text-muted-foreground/80">/ Ops · briefing</p>
                    <h1 className="mt-2 font-[var(--font-display)] text-[40px] font-semibold leading-[0.95] tracking-tight text-foreground sm:text-[52px] lg:text-[60px]">
                        Cast lots.
                    </h1>
                    <p className="mt-3 max-w-prose text-[13.5px] leading-relaxed text-muted-foreground sm:text-sm">
                        Roll a random stage, squad, and challenge modifier. Logged-in doctors can restrict the draw to operators they own and stages they've completed.
                    </p>

                    <div className="mt-5 flex flex-wrap items-center gap-2.5">
                        <Button onClick={onRollAll} size="lg" disabled={!canRoll} className="gap-2 px-5">
                            <Dices aria-hidden="true" className="size-5!" />
                            Roll squad
                        </Button>
                        {hasResult && (
                            <Button onClick={onReset} variant="outline" size="lg">
                                <RotateCcw aria-hidden="true" />
                                Reset
                            </Button>
                        )}
                        <Button onClick={onOpenSettings} variant="ghost" size="lg">
                            <Settings2 aria-hidden="true" />
                            Settings
                        </Button>
                    </div>
                </div>

                <div className="grid w-full grid-cols-3 gap-2 self-end lg:w-[420px]">
                    <StatTile label="Roster" value={operatorsRoster} sub={`of ${operatorsRoster + Math.max(0, operatorsAvailable - operatorsRoster)} known`} />
                    <StatTile label="Drawable ops" value={operatorsAvailable} accent={operatorsAvailable === 0 ? "warning" : "default"} sub={operatorsAvailable === 0 ? "0 match filters" : "match filters"} />
                    <StatTile label="Drawable stages" value={stagesAvailable} accent={stagesAvailable === 0 ? "warning" : "default"} sub={stagesAvailable === 0 ? "0 match filters" : "match filters"} />
                </div>
            </div>
        </section>
    );
}

function Guilloche(): React.ReactElement {
    return (
        <svg aria-hidden="true" className="pointer-events-none absolute inset-x-0 bottom-0 h-8 w-full text-[var(--lagoon)]/30 dark:text-[var(--lagoon)]/20" viewBox="0 0 800 32" preserveAspectRatio="none">
            <path d="M0 16 C 40 4, 80 28, 120 16 S 200 4, 240 16 S 320 28, 360 16 S 440 4, 480 16 S 560 28, 600 16 S 680 4, 720 16 S 800 28, 800 16" stroke="currentColor" strokeWidth="1" fill="none" />
            <path d="M0 16 C 40 28, 80 4, 120 16 S 200 28, 240 16 S 320 4, 360 16 S 440 28, 480 16 S 560 4, 600 16 S 680 28, 720 16 S 800 4, 800 16" stroke="currentColor" strokeWidth="1" fill="none" opacity="0.5" />
        </svg>
    );
}
