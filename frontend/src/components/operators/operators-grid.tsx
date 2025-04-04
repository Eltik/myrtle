import { OperatorRarity, type Operator } from "~/types/impl/api/static/operator";
import Link from "next/link";
import Image from "next/image";
import { formatProfession } from "~/helper";
import { useState } from "react";

/**
 * @author https://wuwatracker.com/resonator
 * A lot of credit to them! They have an amazing design and I copied almost all of it for this.
 * I will be changing it in the future but for now, it's a good placeholder.
 */

export function OperatorsGrid({ operators, currentPage, pageSize }: { operators: Operator[]; currentPage: number; pageSize: number }) {
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);

    // Pre-filter operators to only include those with valid IDs
    const validOperators = operators.filter((operator) => operator.id?.startsWith("char"));

    // Calculate start and end index for current page
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = Math.min(startIndex + pageSize, validOperators.length);

    // Get only the operators for the current page
    const paginatedOperators = validOperators.slice(startIndex, endIndex);

    return (
        <>
            <div className="grid grid-cols-3 gap-2 sm:grid-cols-5 lg:grid-cols-6 lg:gap-3 xl:grid-cols-8 xl:gap-4">
                {paginatedOperators.map((operator) => {
                    const operatorId = operator.id!; // We know it's non-null from our filter
                    const isHovered = hoveredOperator === operatorId;
                    const shouldGrayscale = hoveredOperator !== null && !isHovered;

                    return (
                        <Link href={`/operators?id=${operatorId}`} key={operatorId} className="group relative flex aspect-[2/3] overflow-clip rounded-md border border-muted/50 bg-card transition hover:rounded-lg" onMouseEnter={() => setHoveredOperator(operatorId)} onMouseLeave={() => setHoveredOperator(null)}>
                            <div className="absolute -translate-x-8 -translate-y-4">
                                <Image src={operator.nationId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${String(operator.nationId)}.png` : operator.teamId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${operator.teamId}.png` : `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/none.png`} alt={String(operator.nationId)} loading="lazy" width={360} height={360} decoding="async" className="opacity-5 transition-opacity group-hover:opacity-10" />
                            </div>
                            <div className="absolute inset-0">
                                <div className={`relative h-full w-full scale-100 transition-all duration-150 group-hover:scale-105 ${shouldGrayscale ? "grayscale" : ""} ${isHovered ? "grayscale-0" : ""}`}>
                                    <Image loading="lazy" className="h-full w-full rounded-lg object-contain" alt="Operator Image" src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${operatorId}_1.png`} fill decoding="async" />
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
                                    <div className={`absolute bottom-0 h-0.5 w-full ${operator.rarity === OperatorRarity.sixStar ? "bg-[#f7a452]" : operator.rarity === OperatorRarity.fiveStar ? "bg-[#f7e79e]" : operator.rarity === OperatorRarity.fourStar ? "bg-[#bcabdb]" : operator.rarity === OperatorRarity.threeStar ? "bg-[#88c8e3]" : operator.rarity === OperatorRarity.twoStar ? "bg-[#7ef2a3]" : "bg-white"} ${shouldGrayscale ? "grayscale" : ""} ${isHovered ? "grayscale-0" : ""}`} />
                                    <div className={`absolute -bottom-0.5 h-1 w-full blur-sm ${operator.rarity === OperatorRarity.sixStar ? "bg-[#cc9b6a]" : operator.rarity === OperatorRarity.fiveStar ? "bg-[#d6c474]" : operator.rarity === OperatorRarity.fourStar ? "bg-[#9e87c7]" : operator.rarity === OperatorRarity.threeStar ? "bg-[#62a2bd]" : operator.rarity === OperatorRarity.twoStar ? "bg-[#57ab72]" : "bg-gray-500"}`} />
                                </div>
                            </div>
                        </Link>
                    );
                })}
            </div>
        </>
    );
}
