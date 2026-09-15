import { Dices, RotateCcw, Settings2 } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BriefingHero.messages";

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
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:flex-wrap sm:items-end sm:justify-between sm:gap-4">
            <div className="min-w-0 flex-1">
                <h1 className="m-0 font-bold font-sans text-[22px] text-foreground leading-[1.1] tracking-tight sm:text-[26px] lg:text-[30px]">{t("randomizer.hero.title")}</h1>
                <p className="mt-1.5 max-w-2xl font-sans text-[12.5px] text-muted-foreground leading-normal sm:text-[13.5px]">{t("randomizer.hero.intro")}</p>
                <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground sm:text-[11.5px]">
                    <span>
                        {t("randomizer.hero.roster")}
                        <span className="font-semibold text-foreground">{operatorsRoster}</span>
                    </span>
                    <span aria-hidden="true" className="text-border">
                        ·
                    </span>
                    <span className={cn(operatorsAvailable === 0 && "text-warning")}>
                        {t("randomizer.hero.drawableOps")}
                        <span className="font-semibold text-foreground">{operatorsAvailable}</span>
                    </span>
                    <span aria-hidden="true" className="text-border">
                        ·
                    </span>
                    <span className={cn(stagesAvailable === 0 && "text-warning")}>
                        {t("randomizer.hero.drawableStages")}
                        <span className="font-semibold text-foreground">{stagesAvailable}</span>
                    </span>
                </div>
            </div>
            <div className="flex w-full flex-wrap items-center gap-2 sm:w-auto sm:flex-nowrap">
                <Button onClick={onRollAll} size="sm" disabled={!canRoll} className="flex-1 sm:flex-none">
                    <Dices aria-hidden="true" />
                    {t("randomizer.hero.roll")}
                </Button>
                {hasResult && (
                    <Button onClick={onReset} variant="outline" size="sm" className="flex-1 sm:flex-none">
                        <RotateCcw aria-hidden="true" />
                        {t("randomizer.hero.reset")}
                    </Button>
                )}
                <Button onClick={onOpenSettings} variant="ghost" size="sm" className="flex-1 sm:flex-none">
                    <Settings2 aria-hidden="true" />
                    {t("randomizer.hero.settings")}
                </Button>
            </div>
        </div>
    );
}
