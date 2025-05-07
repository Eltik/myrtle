import type { Dispatch, SetStateAction } from "react";
import { RARITY_COLORS } from "./helper";
import type { Operator } from "~/types/impl/api/static/operator";
import { formatProfession, rarityToNumber } from "~/helper";
import Image from "next/image";
import { cn } from "~/lib/utils";
import { XCircleIcon } from "lucide-react";

// Render helper for operator grid items (Grid View) - Enhanced
export const renderOperatorGridItem = (op: Operator, excludedOperators: Set<string>, setExcludedOperators: Dispatch<SetStateAction<Set<string>>>, lastCharacterRef: ((node: HTMLDivElement) => void) | null) => {
    if (!op.id) return null;

    const isExcluded = excludedOperators.has(op.id);
    const displayRarityColor = RARITY_COLORS[op.rarity] ?? "text-white";
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
        <div key={op.id} ref={lastCharacterRef} className={cn("group relative aspect-[2/3] cursor-pointer overflow-hidden rounded-md border border-muted/50 bg-card shadow-sm transition-all duration-150", isExcluded ? "opacity-50 grayscale" : "hover:border-foreground/50 hover:shadow-md")} onClick={() => handleToggleExclude(op.id)} title={isExcluded ? `Click to Allow ${op.name}` : `Click to Exclude ${op.name}`}>
            <Image src={imageUrl} alt={op.name} fill sizes="(max-width: 640px) 30vw, (max-width: 1024px) 15vw, 10vw" className={cn("object-cover transition-transform duration-150", !isExcluded && "group-hover:scale-105")} loading="lazy" unoptimized />
            <div className="absolute inset-x-0 bottom-0 z-10 bg-black/60 bg-gradient-to-t from-black/80 via-black/50 to-transparent p-2 backdrop-blur-sm">
                <p className={`truncate text-sm font-semibold ${displayRarityColor}`}>{op.name}</p>
                <p className="truncate text-xs text-gray-300">{displayProfession}</p>
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
