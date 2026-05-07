import { Layers, Palette } from "lucide-react";
import { cn } from "#/lib/utils";
import { PALETTE } from "../palette";
import { CARD_PADDING, Kicker, MetricRow, StatCard, Tile } from "../primitives";

interface IModulesSkinsCardProps {
    modules: { unlocked: number; atMax: number; totalAvailable: number };
    skins: { totalOwned: number; totalAvailable: number; percentage: number };
}

export function ModulesSkinsCard({ modules, skins }: IModulesSkinsCardProps) {
    const moduleUnlockPct = modules.totalAvailable > 0 ? (modules.unlocked / modules.totalAvailable) * 100 : 0;

    return (
        <StatCard color={PALETTE.modules.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <div className="space-y-4">
                    <Kicker icon={Layers} label="Modules" />
                    <div className="grid grid-cols-2 gap-2">
                        <Tile
                            color={PALETTE.modules.unlocked}
                            sub="Unlocked"
                            tooltip={`${modules.unlocked} of ${modules.totalAvailable} available modules unlocked`}
                            value={
                                <span>
                                    {modules.unlocked}
                                    <span className="ml-1 text-sm font-medium text-muted-foreground/50">/ {modules.totalAvailable}</span>
                                </span>
                            }
                        />
                        <Tile color={PALETTE.modules.max} sub="Max Lv" tooltip="Modules upgraded to level 3" value={modules.atMax} />
                    </div>
                    <MetricRow color={PALETTE.modules.accent} label="Unlock Rate" pct={moduleUnlockPct} value={`${moduleUnlockPct.toFixed(1)}%`} />
                </div>

                <div className="border-t border-border/60" />

                <div className="space-y-4">
                    <Kicker icon={Palette} label="Skins" />
                    <Tile
                        color={PALETTE.skins}
                        sub="Skins Collected"
                        tooltip={`${skins.totalOwned} of ${skins.totalAvailable} non-default skins collected`}
                        value={
                            <span>
                                {skins.totalOwned}
                                <span className="ml-1 text-sm font-medium text-muted-foreground/50">/ {skins.totalAvailable}</span>
                            </span>
                        }
                    />
                    <MetricRow color={PALETTE.skins} label="Collected" pct={skins.percentage} value={`${skins.percentage.toFixed(1)}%`} />
                </div>
            </div>
        </StatCard>
    );
}
