import { OperatorRarity } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";

export const operatorRarityToNumber = (rarity: OperatorRarity): number => {
    switch (rarity) {
        case OperatorRarity.sixStar:
            return 6;
        case OperatorRarity.fiveStar:
            return 5;
        case OperatorRarity.fourStar:
            return 4;
        case OperatorRarity.threeStar:
            return 3;
        case OperatorRarity.twoStar:
            return 2;
        case OperatorRarity.oneStar:
            return 1;
        default:
            throw new Error(`Unknown operator rarity: ${rarity}`);
    }
};
