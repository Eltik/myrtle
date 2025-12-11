import { OperatorRarity, type Operator } from "~/types/impl/api/static/operator";
import Link from "next/link";
import Image from "next/image";
import { formatProfession, formatSubProfession, capitalize, rarityToNumber } from "~/helper";
import { useState } from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card";

/**
 * @author https://wuwatracker.com/resonator
 * A lot of credit to them! They have an amazing design and I copied almost all of it for this.
 * I will be changing it in the future but for now, it's a good placeholder.
 */

const RarityColors = {
    [OperatorRarity.sixStar]: "#f7a452",
    [OperatorRarity.fiveStar]: "#f7e79e",
    [OperatorRarity.fourStar]: "#bcabdb",
    [OperatorRarity.threeStar]: "#88c8e3",
    [OperatorRarity.twoStar]: "#7ef2a3",
    [OperatorRarity.oneStar]: "#ffffff",
};

const RarityBlurColors = {
    [OperatorRarity.sixStar]: "#cc9b6a",
    [OperatorRarity.fiveStar]: "#d6c474",
    [OperatorRarity.fourStar]: "#9e87c7",
    [OperatorRarity.threeStar]: "#62a2bd",
    [OperatorRarity.twoStar]: "#57ab72",
    [OperatorRarity.oneStar]: "#aaaaaa",
};

export function OperatorsGrid({ operators, currentPage, pageSize }: { operators: Operator[]; currentPage: number; pageSize: number }) {
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);
    const HOVER_DELAY = 500;

    // Pre-filter operators to only include those with valid IDs
    const validOperators = operators.filter((operator) => operator.id?.startsWith("char"));

    // Calculate start and end index for current page
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = Math.min(startIndex + pageSize, validOperators.length);

    // Get only the operators for the current page
    const paginatedOperators = validOperators.slice(startIndex, endIndex);

    const handleOpenChange = (isOpen: boolean, operatorId: string) => {
        if (isOpen) {
            setHoveredOperator(operatorId);
            setIsGrayscaleActive(true);
        } else {
            if (hoveredOperator === operatorId) {
                setHoveredOperator(null);
                setIsGrayscaleActive(false);
            }
        }
    };

    return (
        <>
            <div className="grid grid-cols-3 gap-2 sm:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:grid-cols-8 xl:gap-4">
                {paginatedOperators.map((operator) => {
                    const operatorId = operator.id!;
                    const isCurrentlyHovered = hoveredOperator === operatorId;
                    const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;
                    const rarityColor = RarityColors[operator.rarity] ?? "#ffffff";
                    const rarityBlurColor = RarityBlurColors[operator.rarity] ?? "#aaaaaa";

                    return (
                        <HoverCard key={operatorId} openDelay={HOVER_DELAY} closeDelay={50} onOpenChange={(isOpen) => handleOpenChange(isOpen, operatorId)}>
                            <HoverCardTrigger asChild>
                                <Link href={`/operators?id=${operatorId}`} className="group relative flex aspect-[2/3] overflow-clip rounded-md border border-muted/50 bg-card transition hover:rounded-lg" aria-label={`View details for ${operator.name}`}>
                                    <div className="absolute -translate-x-8 -translate-y-4">
                                        <Image
                                            src={
                                                operator.nationId
                                                    ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${String(operator.nationId)}.png`
                                                    : operator.teamId
                                                      ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${operator.teamId}.png`
                                                      : `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/none.png`
                                            }
                                            alt={String(operator.nationId ?? operator.teamId ?? "Unknown Faction")}
                                            loading="lazy"
                                            width={360}
                                            height={360}
                                            decoding="async"
                                            className="opacity-5 transition-opacity group-hover:opacity-10"
                                        />
                                    </div>
                                    <div className="absolute inset-0">
                                        <div className={`relative h-full w-full scale-100 transition-all duration-150 group-hover:scale-105 ${shouldGrayscale ? "grayscale" : ""} ${isCurrentlyHovered ? "grayscale-0" : ""}`}>
                                            <Image loading="lazy" className="h-full w-full rounded-lg object-contain" alt={`${operator.name} Portrait`} src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${operatorId}_1.png`} fill decoding="async" />
                                        </div>
                                    </div>
                                    <div className="absolute inset-x-0 bottom-0 z-10">
                                        <div className="relative">
                                            <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                                            <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[85%] pr-8 text-xs font-bold uppercase opacity-60 transition-opacity group-hover:opacity-100 sm:text-sm md:text-sm">{operator.name}</h2>
                                            <div className="absolute bottom-1 right-1 flex scale-75 items-center opacity-0 transition-all duration-200 group-hover:scale-100 group-hover:opacity-100">
                                                <div className="h-4 w-4 md:h-6 md:w-6">
                                                    <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatProfession(operator.profession).toLowerCase()}.png`} alt={formatProfession(operator.profession)} loading="lazy" width={160} height={160} decoding="async" />
                                                </div>
                                            </div>
                                            <div className={`absolute bottom-0 h-0.5 w-full ${shouldGrayscale ? "grayscale" : ""} ${isCurrentlyHovered ? "grayscale-0" : ""}`} style={{ backgroundColor: rarityColor }} />
                                            <div className={`absolute -bottom-0.5 h-1 w-full blur-sm ${shouldGrayscale ? "grayscale" : ""} ${isCurrentlyHovered ? "grayscale-0" : ""}`} style={{ backgroundColor: rarityBlurColor }} />
                                        </div>
                                    </div>
                                </Link>
                            </HoverCardTrigger>
                            <HoverCardContent className="w-80 p-4" side="top">
                                <div className="flex items-start space-x-4">
                                    <div className="relative h-16 w-16 flex-shrink-0">
                                        <Image
                                            src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${operatorId}.png`}
                                            alt={`${operator.name} Avatar`}
                                            fill
                                            className="rounded-md object-cover"
                                            unoptimized // Might be needed if domain isn't configured in next.config.js
                                        />
                                    </div>
                                    <div className="flex-grow space-y-1">
                                        <div className="flex items-center justify-between">
                                            <h4 className="text-base font-semibold">{operator.name}</h4>
                                            <div className="h-6 w-6 flex-shrink-0">
                                                <Image
                                                    src={
                                                        operator.nationId
                                                            ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${String(operator.nationId)}.png`
                                                            : operator.teamId
                                                              ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${operator.teamId}.png`
                                                              : `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_rhodes.png`
                                                    } // Default to Rhodes Island if no nation/team
                                                    alt={String(operator.nationId ?? operator.teamId ?? "Rhodes Island")}
                                                    width={24}
                                                    height={24}
                                                    className="object-contain"
                                                    unoptimized // Might be needed
                                                />
                                            </div>
                                        </div>
                                        <p className="text-xs font-semibold" style={{ color: rarityColor }}>
                                            {`${rarityToNumber(operator.rarity)}★ ${capitalize(formatSubProfession(operator.subProfessionId.toLowerCase()))} ${formatProfession(operator.profession)}`}
                                        </p>
                                        <div className="flex space-x-2 pt-1 text-xs text-muted-foreground">
                                            <span>{capitalize(operator.position?.toLowerCase() ?? "Unknown")}</span>
                                            {operator.profile?.basicInfo?.race && (
                                                <>
                                                    <span>•</span>
                                                    <span>{operator.profile.basicInfo.race}</span>
                                                </>
                                            )}
                                            {operator.tagList && operator.tagList.length > 0 && (
                                                <>
                                                    <span>•</span>
                                                    <span className="truncate" title={operator.tagList.join(", ")}>
                                                        {operator.tagList[0]} {/* Show first tag */}
                                                    </span>
                                                </>
                                            )}
                                        </div>
                                        {/* Removed description as requested */}
                                    </div>
                                </div>
                            </HoverCardContent>
                        </HoverCard>
                    );
                })}
            </div>
        </>
    );
}
