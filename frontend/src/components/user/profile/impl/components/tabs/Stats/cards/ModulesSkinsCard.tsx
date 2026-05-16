import { Layers, Palette } from "lucide-react";
import { useMemo } from "react";
import type { ISkin } from "#/lib/api/skins";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IModuleGapDetails } from "../helpers";
import { PALETTE } from "../palette";
import { CARD_PADDING, GapList, type IGapItem, Kicker, MetricRow, StatCard, Tile } from "../primitives";
import { SkinViewerDialog } from "./SkinViewerDialog";

interface IModulesSkinsCardProps {
    modules: { unlocked: number; atMax: number; totalAvailable: number; details: IModuleGapDetails };
    skins: { totalOwned: number; totalAvailable: number; percentage: number };
    charSkins: Record<string, ISkin>;
    ownedSkinIds: Set<string>;
    operatorsStatic: IOperatorListItem[];
}

export function ModulesSkinsCard({ modules, skins, charSkins, ownedSkinIds, operatorsStatic }: IModulesSkinsCardProps) {
    const operatorsMap = useMemo(() => {
        const map = new Map<string, IOperatorListItem>();
        for (const op of operatorsStatic) {
            if (op.id) map.set(op.id, op);
        }
        return map;
    }, [operatorsStatic]);

    const allSkins = useMemo(() => Object.values(charSkins), [charSkins]);
    const moduleUnlockPct = modules.totalAvailable > 0 ? (modules.unlocked / modules.totalAvailable) * 100 : 0;

    const moduleGaps: IGapItem[] = [
        {
            key: "locked",
            label: "locked",
            value: Math.max(0, modules.totalAvailable - modules.unlocked),
            color: PALETTE.modules.unlocked,
            tooltip: "Click to view modules available on owned operators but not unlocked",
            details: modules.details.locked,
        },
        {
            key: "belowMax",
            label: "below max",
            value: Math.max(0, modules.unlocked - modules.atMax),
            color: PALETTE.modules.max,
            tooltip: "Click to view unlocked modules that are not yet at level 3",
            details: modules.details.belowMax,
        },
    ];

    const skinGaps: IGapItem[] = [
        {
            key: "missing",
            label: "missing",
            value: Math.max(0, skins.totalAvailable - skins.totalOwned),
            color: PALETTE.skins,
            tooltip: "Open the skin collection viewer",
            dialogContent: <SkinViewerDialog color={PALETTE.skins} operatorsMap={operatorsMap} ownedIds={ownedSkinIds} profileOwnedCount={skins.totalOwned} skins={allSkins} />,
        },
    ];

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
                                    <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ {modules.totalAvailable}</span>
                                </span>
                            }
                        />
                        <Tile color={PALETTE.modules.max} sub="Max Lv" tooltip="Modules upgraded to level 3" value={modules.atMax} />
                    </div>
                    <MetricRow color={PALETTE.modules.accent} label="Unlock Rate" pct={moduleUnlockPct} value={`${moduleUnlockPct.toFixed(1)}%`} />
                    <GapList items={moduleGaps} />
                </div>

                <div className="border-border/60 border-t" />

                <div className="space-y-4">
                    <Kicker icon={Palette} label="Skins" />
                    <Tile
                        color={PALETTE.skins}
                        sub="Skins Collected"
                        tooltip={`${skins.totalOwned} of ${skins.totalAvailable} non-default skins collected`}
                        value={
                            <span>
                                {skins.totalOwned}
                                <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ {skins.totalAvailable}</span>
                            </span>
                        }
                    />
                    <MetricRow color={PALETTE.skins} label="Collected" pct={skins.percentage} value={`${skins.percentage.toFixed(1)}%`} />
                    <GapList items={skinGaps} />
                </div>
            </div>
        </StatCard>
    );
}
