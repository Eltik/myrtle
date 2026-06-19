export const PALETTE = {
    collection: "var(--primary)",
    elite: "oklch(0.74 0.17 75)",
    top: "var(--primary)",
    signin: "oklch(0.70 0.15 162)",

    classes: {
        PIONEER: "oklch(0.70 0.14 200)",
        WARRIOR: "oklch(0.62 0.22 25)",
        TANK: "oklch(0.74 0.16 75)",
        SNIPER: "oklch(0.62 0.20 255)",
        CASTER: "oklch(0.62 0.22 295)",
        SUPPORT: "oklch(0.70 0.16 145)",
        MEDIC: "oklch(0.78 0.16 110)",
        SPECIAL: "oklch(0.65 0.22 340)",
    } as Record<string, string>,

    mastery: {
        accent: "oklch(0.62 0.21 295)",
        m3: "oklch(0.70 0.17 145)",
        m6: "oklch(0.65 0.18 230)",
        m9: "oklch(0.74 0.17 75)",
    },

    modules: {
        accent: "oklch(0.66 0.14 200)",
        unlocked: "oklch(0.66 0.14 200)",
        max: "oklch(0.72 0.15 175)",
    },
    skins: "var(--primary)",
} as const;
