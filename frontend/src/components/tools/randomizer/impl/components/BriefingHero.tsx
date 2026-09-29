import { Dices, RotateCcw, Settings2 } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/ui/page-header";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./BriefingHero.messages";

interface IBriefingHeroProps {
    breadcrumb: readonly React.ReactNode[];
    operatorsAvailable: number;
    operatorsRoster: number;
    stagesAvailable: number;
    hasResult: boolean;
    canRoll: boolean;
    onRollAll: () => void;
    onReset: () => void;
    onOpenSettings: () => void;
}

export function BriefingHero({ breadcrumb, operatorsAvailable, operatorsRoster, stagesAvailable, hasResult, canRoll, onRollAll, onReset, onOpenSettings }: IBriefingHeroProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <div className="flex flex-col">
            <PageHeader
                breadcrumbLabel="breadcrumb"
                breadcrumb={breadcrumb}
                title={t("randomizer.hero.title")}
                description={t("randomizer.hero.intro")}
                actions={
                    <div className="flex flex-wrap items-center gap-2 sm:flex-nowrap">
                        <Button onClick={onRollAll} size="sm" disabled={!canRoll}>
                            <Dices aria-hidden="true" />
                            {t("randomizer.hero.roll")}
                        </Button>
                        {hasResult && (
                            <Button onClick={onReset} variant="outline" size="sm">
                                <RotateCcw aria-hidden="true" />
                                {t("randomizer.hero.reset")}
                            </Button>
                        )}
                        <Button onClick={onOpenSettings} variant="ghost" size="sm">
                            <Settings2 aria-hidden="true" />
                            {t("randomizer.hero.settings")}
                        </Button>
                    </div>
                }
            />
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
    );
}
