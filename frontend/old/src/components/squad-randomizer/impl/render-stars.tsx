import { StarIcon } from "lucide-react";
import { rarityToNumber } from "~/helper";
import type { OperatorRarity } from "~/types/impl/api/static/operator";
import { RARITY_COLORS } from "./helper";

// Helper to get star icons
export const renderStars = (rarityEnum: OperatorRarity) => {
    const rarityNum = rarityToNumber(rarityEnum);
    return (
        <div className="flex items-center">
            {Array.from({ length: rarityNum }, (_, i) => (
                <StarIcon className={`h-3 w-3 ${RARITY_COLORS[rarityEnum]?.replace("text-", "fill-") ?? "fill-gray-400"}`} key={i} />
            ))}
        </div>
    );
};
