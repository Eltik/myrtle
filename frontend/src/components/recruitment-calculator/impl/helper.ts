export const PROFESSION_TO_TAG_ID: Record<string, string> = {
    Guard: "1",
    Sniper: "2",
    Defender: "3",
    Medic: "4",
    Supporter: "5",
    Caster: "6",
    Specialist: "7",
    Vanguard: "8",
};

// Helper map for position to tag ID
export const POSITION_TO_TAG_ID: Record<string, string> = {
    MELEE: "9",
    RANGED: "10",
};

// Helper map for rarity to tag ID
export const RARITY_TO_TAG_ID: Record<number, string> = {
    0: "28", // Robot
    1: "17", // Starter (1*)
    4: "14", // Senior Op
    5: "11", // Top Op
};

export const getRarityColor = (rarityNum: number): string => {
    if (rarityNum === 6) return "text-orange-500 font-bold";
    if (rarityNum === 5) return "text-yellow-500 font-semibold";
    if (rarityNum === 4) return "text-purple-500 font-semibold";
    if (rarityNum === 3) return "text-blue-500";
    if (rarityNum === 2) return "text-green-600";
    if (rarityNum === 1) return "text-gray-500";
    return "text-gray-400";
};

// Helper function to get rarity border color
export const getRarityBorderColor = (rarityNum: number): string => {
    if (rarityNum === 6) return "border border-orange-400/60 shadow-[0_0_6px_theme(colors.orange.400/50)]"; // Top Op
    if (rarityNum === 5) return "border border-yellow-400/60 shadow-[0_0_6px_theme(colors.yellow.400/50)]"; // Senior Op
    if (rarityNum === 4) return "border border-purple-400/60 shadow-[0_0_6px_theme(colors.purple.400/50)]"; // 4*
    if (rarityNum === 3) return "border border-blue-400/60 shadow-[0_0_6px_theme(colors.blue.400/50)]"; // 3*
    if (rarityNum === 2) return "border border-green-500/60 shadow-[0_0_6px_theme(colors.green.500/50)]"; // 2*
    if (rarityNum === 1) return "border border-gray-400/60 shadow-[0_0_6px_theme(colors.gray.400/50)]"; // 1*
    return "border border-gray-300/60 shadow-[0_0_6px_theme(colors.gray.300/50)]"; // Default/Robot
};
