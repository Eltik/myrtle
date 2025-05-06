import { formatSubProfession } from "~/helper";

import { formatProfession } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";
import { RARITY_COLORS } from "./helper";
import Image from "next/image";
import { cn } from "~/lib/utils";
import { renderStars } from "./render-stars";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import type { Dispatch, SetStateAction } from "react";

// Updated List View Renderer
export const renderOperatorListItem = (op: Operator, excludedOperators: Set<string>, setExcludedOperators: Dispatch<SetStateAction<Set<string>>>) => {
    if (!op.id) return null;
    const isExcluded = excludedOperators.has(op.id);
    const displayRarityColor = RARITY_COLORS[op.rarity] ?? "text-white";
    const displayProfession = formatProfession(op.profession);
    const displaySubProfession = formatSubProfession(op.subProfessionId);
    const imageUrl = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${op.id}.png`;

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
        <div key={op.id} className={cn("flex items-center border-b p-2 last:border-b-0", isExcluded && "bg-muted/30 opacity-60")}>
            <div className="mr-2 flex min-w-0 items-center space-x-3">
                <Image src={imageUrl} alt={op.name} width={40} height={40} className={cn("flex-shrink-0 rounded-full", isExcluded && "grayscale")} unoptimized />
                <div className="min-w-0 overflow-hidden">
                    <p className={`truncate font-semibold ${displayRarityColor}`}>{op.name}</p>
                    <p className="truncate text-xs text-muted-foreground">
                        {renderStars(op.rarity)} {displaySubProfession} <span className="mx-1">|</span> {displayProfession}
                    </p>
                    <div className="mt-1 flex flex-wrap gap-1">
                        {op.tagList?.slice(0, 4).map((tag) => (
                            <Badge key={tag} variant="secondary" className="px-1 py-0 text-xs">
                                {tag}
                            </Badge>
                        ))}
                    </div>
                </div>
            </div>
            <div className="ml-auto flex flex-shrink-0 items-center space-x-1 sm:space-x-2">
                <Button
                    variant={isExcluded ? "secondary" : "outline"}
                    size="sm"
                    onClick={() => {
                        handleToggleExclude(op.id);
                    }}
                    title={isExcluded ? "Allow in squad" : "Exclude from squad"}
                    className={cn("h-auto px-2 py-1 text-xs", isExcluded ? "text-muted-foreground hover:bg-muted hover:text-foreground" : "text-destructive hover:bg-destructive/10")}
                >
                    {isExcluded ? "Allow" : "Exclude"}
                </Button>
            </div>
        </div>
    );
};
