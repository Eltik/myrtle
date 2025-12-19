import Image from "next/image";
import Link from "next/link";
import { memo } from "react";
import { capitalize, cn, formatProfession, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { RARITY_COLORS } from "../../constants";
import { ClassIcon } from "./class-icon";
import { FactionLogo } from "./faction-logo";
import { RarityStars } from "./rarity-stars";

interface OperatorCardListProps {
    operator: OperatorFromList;
    listColumns?: number;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
}

export const OperatorCardList = memo(function OperatorCardList({ operator, listColumns = 1, isHovered = false, shouldGrayscale = false }: OperatorCardListProps) {
    // Use compact layout when displaying in multi-column grid
    const useCompactLayout = listColumns > 1;
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const operatorId = operator.id ?? "";

    return (
        <Link className={cn("group card-hover-transition relative flex items-center gap-3 rounded-lg border border-transparent bg-card/50 px-3 py-2.5 grayscale-transition contain-layout hover:border-border hover:bg-card", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} href={`/operators?id=${operatorId}`}>
            {/* Rarity indicator line on left */}
            <div className="-translate-y-1/2 absolute top-1/2 left-0 h-8 w-0.5 rounded-full opacity-60 opacity-transition group-hover:opacity-100" style={{ backgroundColor: rarityColor }} />

            {/* Portrait */}
            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                <div className="absolute inset-0 transform-gpu transition-transform duration-200 ease-out group-hover:scale-110">
                    <Image alt={operator.name} className="object-cover" fill src={`/api/cdn${operator.portrait}`} />
                </div>
            </div>

            {/* Desktop Layout: Name column - only show when not in compact layout */}
            {!useCompactLayout && (
                <div className="hidden min-w-0 flex-1 md:block">
                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                </div>
            )}

            {/* Desktop: Rarity stars - only show when not in compact layout */}
            {!useCompactLayout && <RarityStars className="hidden w-24 shrink-0 items-center gap-0.5 md:flex" rarity={rarityNum} starClassName="text-sm" />}

            {/* Desktop: Class info - only show when not in compact layout */}
            {!useCompactLayout && (
                <div className="hidden w-32 shrink-0 items-center gap-2 md:flex">
                    <div className="flex h-5 w-5 items-center justify-center">
                        <ClassIcon className="opacity-60 transition-opacity group-hover:opacity-100" profession={operator.profession} size={20} />
                    </div>
                    <span className="text-muted-foreground text-sm">{formatProfession(operator.profession)}</span>
                </div>
            )}

            {/* Desktop: Archetype - only show when not in compact layout */}
            {!useCompactLayout && (
                <div className="hidden w-40 shrink-0 lg:block">
                    <span className="truncate text-muted-foreground text-sm">{capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))}</span>
                </div>
            )}

            {/* Desktop: Faction logo - only show when not in compact layout */}
            {!useCompactLayout && (
                <div className="hidden w-8 shrink-0 justify-center xl:flex">
                    <div className="flex h-6 w-6 items-center justify-center opacity-40 transition-opacity group-hover:opacity-70">
                        <FactionLogo className="object-contain" nationId={operator.nationId} size={24} teamId={operator.teamId} />
                    </div>
                </div>
            )}

            {/* Compact/Mobile Layout - show when compact layout or on mobile */}
            <div className={cn("flex min-w-0 flex-1 flex-col gap-1", useCompactLayout ? "flex" : "md:hidden")}>
                {/* Name row with faction icon */}
                <div className="flex items-center gap-2">
                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{operator.name}</span>
                    <div className="flex h-4 w-4 shrink-0 items-center justify-center opacity-50">
                        <FactionLogo className="object-contain" nationId={operator.nationId} size={16} teamId={operator.teamId} />
                    </div>
                </div>

                {/* Info row: Rarity, Class, Archetype */}
                <div className="flex items-center gap-2 text-xs">
                    <RarityStars className="flex items-center gap-0.5" rarity={rarityNum} />

                    <span className="text-muted-foreground/50">·</span>

                    {/* Class with icon */}
                    <div className="flex items-center gap-1">
                        <div className="flex h-4 w-4 items-center justify-center">
                            <ClassIcon className="opacity-60" profession={operator.profession} size={14} />
                        </div>
                        <span className="text-muted-foreground">{formatProfession(operator.profession)}</span>
                    </div>

                    <span className="text-muted-foreground/50">·</span>

                    {/* Archetype */}
                    <span className="truncate text-muted-foreground">{capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))}</span>
                </div>
            </div>
        </Link>
    );
});
