import { Dices, RotateCcw, Settings2 } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

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

export function BriefingHero({ operatorsAvailable, operatorsRoster, stagesAvailable, hasResult, canRoll, onRollAll, onReset, onOpenSettings }: IBriefingHeroProps): React.ReactElement {
    return (
        <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:flex-wrap sm:items-end sm:justify-between sm:gap-4">
            <div className="min-w-0 flex-1">
                <h1 className="m-0 font-bold font-sans text-[22px] leading-[1.1] tracking-tight text-foreground sm:text-[26px] lg:text-[30px]">Randomizer</h1>
                <p className="mt-1.5 max-w-2xl font-sans text-[12.5px] leading-normal text-muted-foreground sm:text-[13.5px]">Roll a random stage, squad, and challenge modifier. Logged-in users can restrict the draw to operators they own and stages they've completed.</p>
                <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground sm:text-[11.5px]">
                    <span>
                        Roster <span className="font-semibold text-foreground">{operatorsRoster}</span>
                    </span>
                    <span aria-hidden="true" className="text-border">
                        ·
                    </span>
                    <span className={cn(operatorsAvailable === 0 && "text-warning")}>
                        Drawable ops <span className="font-semibold text-foreground">{operatorsAvailable}</span>
                    </span>
                    <span aria-hidden="true" className="text-border">
                        ·
                    </span>
                    <span className={cn(stagesAvailable === 0 && "text-warning")}>
                        Drawable stages <span className="font-semibold text-foreground">{stagesAvailable}</span>
                    </span>
                </div>
            </div>
            <div className="flex w-full flex-wrap items-center gap-2 sm:w-auto sm:flex-nowrap">
                <Button onClick={onRollAll} size="sm" disabled={!canRoll} className="flex-1 sm:flex-none">
                    <Dices aria-hidden="true" />
                    Roll squad
                </Button>
                {hasResult && (
                    <Button onClick={onReset} variant="outline" size="sm" className="flex-1 sm:flex-none">
                        <RotateCcw aria-hidden="true" />
                        Reset
                    </Button>
                )}
                <Button onClick={onOpenSettings} variant="ghost" size="sm" className="flex-1 sm:flex-none">
                    <Settings2 aria-hidden="true" />
                    Settings
                </Button>
            </div>
        </div>
    );
}
