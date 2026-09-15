import { Layers, Palette } from "lucide-react";
import { useMemo } from "react";
import type { ISkinIndexEntry } from "#/lib/api/skins";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IModuleGapDetails } from "../helpers";
import { PALETTE } from "../palette";
import { CARD_PADDING, GapList, type IGapItem, Kicker, MetricRow, StatCard, Tile } from "../primitives";
import type { messages } from "./ModulesSkinsCard.messages";
import { SkinViewerDialog } from "./SkinViewerDialog";

interface IModulesSkinsCardProps {
    modules: { unlocked: number; atMax: number; totalAvailable: number; details: IModuleGapDetails };
    skins: { totalOwned: number; totalAvailable: number; percentage: number };
    charSkins: Record<string, ISkinIndexEntry>;
    ownedSkinIds: Set<string>;
    operatorsStatic: IOperatorListItem[];
}

export function ModulesSkinsCard({ modules, skins, charSkins, ownedSkinIds, operatorsStatic }: IModulesSkinsCardProps) {
    const t: TypedT<typeof messages> = useT("user");
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
            label: t("profile.stats.modules.gap.locked"),
            value: Math.max(0, modules.totalAvailable - modules.unlocked),
            color: PALETTE.modules.unlocked,
            tooltip: t("profile.stats.modules.gap.locked.tooltip"),
            details: modules.details.locked,
        },
        {
            key: "belowMax",
            label: t("profile.stats.modules.gap.belowMax"),
            value: Math.max(0, modules.unlocked - modules.atMax),
            color: PALETTE.modules.max,
            tooltip: t("profile.stats.modules.gap.belowMax.tooltip"),
            details: modules.details.belowMax,
        },
    ];

    const skinGaps: IGapItem[] = [
        {
            key: "missing",
            label: t("profile.stats.skins.gap.missing"),
            value: Math.max(0, skins.totalAvailable - skins.totalOwned),
            color: PALETTE.skins,
            tooltip: t("profile.stats.skins.gap.missing.tooltip"),
            dialogContent: <SkinViewerDialog color={PALETTE.skins} operatorsMap={operatorsMap} ownedIds={ownedSkinIds} profileOwnedCount={skins.totalOwned} skins={allSkins} />,
        },
    ];

    return (
        <StatCard color={PALETTE.modules.accent}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <div className="space-y-4">
                    <Kicker icon={Layers} label={t("profile.stats.modules.title")} />
                    <div className="grid grid-cols-2 gap-2">
                        <Tile
                            color={PALETTE.modules.unlocked}
                            sub={t("profile.stats.modules.unlocked")}
                            tooltip={t("profile.stats.modules.unlocked.tooltip", { unlocked: modules.unlocked, total: modules.totalAvailable })}
                            value={
                                <span>
                                    {modules.unlocked}
                                    <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ {modules.totalAvailable}</span>
                                </span>
                            }
                        />
                        <Tile color={PALETTE.modules.max} sub={t("profile.stats.modules.maxLevel")} tooltip={t("profile.stats.modules.maxLevel.tooltip")} value={modules.atMax} />
                    </div>
                    <MetricRow color={PALETTE.modules.accent} label={t("profile.stats.modules.unlockRate")} pct={moduleUnlockPct} value={`${moduleUnlockPct.toFixed(1)}%`} />
                    <GapList items={moduleGaps} />
                </div>

                <div className="border-border/60 border-t" />

                <div className="space-y-4">
                    <Kicker icon={Palette} label={t("profile.stats.skins.title")} />
                    <Tile
                        color={PALETTE.skins}
                        sub={t("profile.stats.skins.collected")}
                        tooltip={t("profile.stats.skins.collected.tooltip", { owned: skins.totalOwned, total: skins.totalAvailable })}
                        value={
                            <span>
                                {skins.totalOwned}
                                <span className="ml-1 font-medium text-muted-foreground/50 text-sm">/ {skins.totalAvailable}</span>
                            </span>
                        }
                    />
                    <MetricRow color={PALETTE.skins} label={t("profile.stats.skins.rate")} pct={skins.percentage} value={`${skins.percentage.toFixed(1)}%`} />
                    <GapList items={skinGaps} />
                </div>
            </div>
        </StatCard>
    );
}
