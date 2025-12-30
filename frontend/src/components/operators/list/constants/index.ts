// Rarity color mappings
export const RARITY_COLORS: Record<number, string> = {
    6: "#f7a452",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
    2: "#7ef2a3",
    1: "#ffffff",
};

export const RARITY_BLUR_COLORS: Record<number, string> = {
    6: "#cc9b6a",
    5: "#d6c474",
    4: "#9e87c7",
    3: "#62a2bd",
    2: "#57ab72",
    1: "#aaaaaa",
};

// Internal profession names to display names
export const CLASS_DISPLAY: Record<string, string> = {
    WARRIOR: "Guard",
    SNIPER: "Sniper",
    TANK: "Defender",
    MEDIC: "Medic",
    SUPPORT: "Supporter",
    CASTER: "Caster",
    SPECIAL: "Specialist",
    PIONEER: "Vanguard",
};

// Internal profession names to icon file names
export const CLASS_ICON: Record<string, string> = {
    WARRIOR: "warrior",
    SNIPER: "sniper",
    TANK: "tank",
    MEDIC: "medic",
    SUPPORT: "support",
    CASTER: "caster",
    SPECIAL: "special",
    PIONEER: "pioneer",
};

// Nation ID to display name mappings
export const NATION_DISPLAY: Record<string, string> = {
    rhodes: "Rhodes Island",
    kazimierz: "Kazimierz",
    columbia: "Columbia",
    laterano: "Laterano",
    victoria: "Victoria",
    sami: "Sami",
    bolivar: "Bolivar",
    iberia: "Iberia",
    siracusa: "Siracusa",
    higashi: "Higashi",
    sargon: "Sargon",
    kjerag: "Kjerag",
    minos: "Minos",
    yan: "Yan",
    lungmen: "Lungmen",
    ursus: "Ursus",
    egir: "Ægir",
    leithanien: "Leithanien",
    rim: "Rim Billiton",
};

// Filter option constants
export const CLASSES = ["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER", "SPECIAL", "PIONEER"] as const;
export const RARITIES = [6, 5, 4, 3, 2, 1] as const;
export const GENDERS = ["Male", "Female", "Conviction"] as const;

// Class sort order (for secondary sorting when rarity is the same)
// Order: Vanguard > Guard > Defender > Sniper > Caster > Supporter > Medic > Specialist
export const CLASS_SORT_ORDER: Record<string, number> = {
    PIONEER: 0, // Vanguard
    WARRIOR: 1, // Guard
    TANK: 2, // Defender
    SNIPER: 3,
    CASTER: 4,
    SUPPORT: 5, // Supporter
    MEDIC: 6,
    SPECIAL: 7, // Specialist
};

// Sort options with display names
export const SORT_OPTIONS = [
    { value: "rarity", label: "Rarity" },
    { value: "name", label: "Name" },
    { value: "class", label: "Class" },
    { value: "hp", label: "HP" },
    { value: "atk", label: "ATK" },
    { value: "def", label: "DEF" },
    { value: "res", label: "RES" },
    { value: "cost", label: "Cost" },
    { value: "block", label: "Block" },
] as const;

// Pagination
export const ITEMS_PER_PAGE = 48;

// Hover delay for operator cards
export const HOVER_DELAY = 500;
