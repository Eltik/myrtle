export const SCORE_PALETTE = {
    overall: "var(--primary)",

    operator: "oklch(0.74 0.17 75)",
    base: "oklch(0.70 0.16 145)",
    stage: "oklch(0.62 0.20 255)",
    roguelike: "oklch(0.65 0.22 340)",
    sandbox: "oklch(0.70 0.14 200)",
    medal: "oklch(0.62 0.22 295)",
} as const;

export const GRADE_COLORS: Record<string, string> = {
    "S+": "oklch(0.78 0.18 75)",
    S: "oklch(0.74 0.17 75)",
    A: "oklch(0.70 0.16 145)",
    B: "oklch(0.66 0.14 200)",
    C: "oklch(0.62 0.20 255)",
    D: "oklch(0.65 0.22 295)",
    F: "oklch(0.62 0.22 25)",
};

export function gradeColor(grade: string | null | undefined): string {
    if (!grade) return "oklch(0.55 0.02 285)";
    return GRADE_COLORS[grade] ?? GRADE_COLORS.F;
}
