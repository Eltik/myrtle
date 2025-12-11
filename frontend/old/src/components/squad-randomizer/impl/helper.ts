import { OperatorProfession, OperatorRarity } from "~/types/impl/api/static/operator";

// Use OperatorProfession enum keys
export const PROFESSION_MAP: Record<OperatorProfession, string> = {
    [OperatorProfession.VANGUARD]: "Vanguard",
    [OperatorProfession.GUARD]: "Guard",
    [OperatorProfession.SNIPER]: "Sniper",
    [OperatorProfession.DEFENDER]: "Defender",
    [OperatorProfession.MEDIC]: "Medic",
    [OperatorProfession.SUPPORTER]: "Supporter",
    [OperatorProfession.CASTER]: "Caster",
    [OperatorProfession.SPECIALIST]: "Specialist",
};

// Map OperatorRarity enum to display colors
export const RARITY_COLORS: Record<OperatorRarity, string> = {
    [OperatorRarity.oneStar]: "text-gray-500",
    [OperatorRarity.twoStar]: "text-green-600",
    [OperatorRarity.threeStar]: "text-blue-500",
    [OperatorRarity.fourStar]: "text-purple-500",
    [OperatorRarity.fiveStar]: "text-yellow-500",
    [OperatorRarity.sixStar]: "text-orange-500",
};

export const getRarityBorderColor = (rarityNum: number): string => {
    if (rarityNum === 6) return "border border-orange-400/60 shadow-[0_0_6px_theme(colors.orange.400/50)]"; // Top Op
    if (rarityNum === 5) return "border border-yellow-400/60 shadow-[0_0_6px_theme(colors.yellow.400/50)]"; // Senior Op
    if (rarityNum === 4) return "border border-purple-400/60 shadow-[0_0_6px_theme(colors.purple.400/50)]"; // 4*
    if (rarityNum === 3) return "border border-blue-400/60 shadow-[0_0_6px_theme(colors.blue.400/50)]"; // 3*
    if (rarityNum === 2) return "border border-green-500/60 shadow-[0_0_6px_theme(colors.green.500/50)]"; // 2*
    if (rarityNum === 1) return "border border-gray-400/60 shadow-[0_0_6px_theme(colors.gray.400/50)]"; // 1*
    return "border border-gray-300/60 shadow-[0_0_6px_theme(colors.gray.300/50)]"; // Default/Robot
};
