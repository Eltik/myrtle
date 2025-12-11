import { XCircleIcon } from "lucide-react";
import Image from "next/image";
import type { Dispatch, SetStateAction } from "react";
import { formatProfession, rarityToNumber } from "~/helper";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/impl/api/static/operator";
import { getRarityBorderColor } from "./helper";

// Render helper for operator grid items (Grid View) - Enhanced
export const renderOperatorGridItem = (op: Operator, excludedOperators: Set<string>, setExcludedOperators: Dispatch<SetStateAction<Set<string>>>, lastCharacterRef: ((node: HTMLDivElement) => void) | null) => {
    if (!op.id) return null;

    const isExcluded = excludedOperators.has(op.id);
    const displayProfession = formatProfession(op.profession);
    const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${op.id}_1.png`;
    const rarityNum = rarityToNumber(op.rarity);

    const handleToggleExclude = (operatorId: string | undefined) => {
        if (!operatorId) return; // Ignore if id is undefined
        setExcludedOperators((prev) => {
            const newSet = new Set(prev);
            if (newSet.has(operatorId)) {
                newSet.delete(operatorId);
            } else {
                newSet.add(operatorId);
            }
            return newSet;
        });
    };

    return (
        <div
            className={cn("group relative aspect-[2/3] cursor-pointer overflow-hidden rounded-md border bg-card shadow-sm transition-all duration-150", getRarityBorderColor(rarityNum), isExcluded ? "opacity-50 grayscale" : "hover:border-foreground/50 hover:shadow-md")}
            key={op.id}
            onClick={() => handleToggleExclude(op.id)}
            ref={lastCharacterRef}
            title={isExcluded ? `Click to Allow ${op.name}` : `Click to Exclude ${op.name}`}
        >
            <Image alt={op.name} className={cn("object-cover transition-transform duration-150", !isExcluded && "group-hover:scale-105")} fill loading="lazy" sizes="(max-width: 640px) 30vw, (max-width: 1024px) 15vw, 10vw" src={imageUrl} unoptimized />
            <div className="absolute inset-x-0 bottom-0 z-10 bg-black/60 bg-gradient-to-t from-black/80 via-black/50 to-transparent p-2 backdrop-blur-sm">
                <p className="truncate font-semibold text-sm">{op.name}</p>
                <p className="truncate text-gray-300 text-xs">{displayProfession}</p>
                <div className="text-xs text-yellow-300">{Array(rarityNum).fill("★").join("")}</div>
            </div>
            {isExcluded && (
                <div className="absolute inset-0 z-20 flex items-center justify-center bg-black/40">
                    <XCircleIcon className="h-8 w-8 text-destructive opacity-80" />
                </div>
            )}
        </div>
    );
};
