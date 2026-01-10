export const SORT_OPTIONS = [
    { value: "total_score", label: "Total Score", tooltip: "Overall score combining all categories" },
    { value: "grade", label: "Grade", tooltip: "Letter grade based on percentile ranking" },
    { value: "operator_score", label: "Operators", tooltip: "Score from operator collection and investment" },
    { value: "stage_score", label: "Stages", tooltip: "Score from stage completions and challenges" },
    { value: "roguelike_score", label: "Roguelike", tooltip: "Integrated Strategies progress" },
    { value: "sandbox_score", label: "Sandbox", tooltip: "Reclamation Algorithm progress" },
    { value: "medal_score", label: "Medals", tooltip: "Medal collection progress" },
    { value: "base_score", label: "Base", tooltip: "Rhodes Island base development" },
    { value: "composite_score", label: "Composite", tooltip: "Weighted combination score" },
] as const;

export const SERVERS = [
    { value: "all", label: "All Servers" },
    { value: "en", label: "Global (EN)" },
    { value: "jp", label: "Japan" },
    { value: "cn", label: "China" },
    { value: "kr", label: "Korea" },
    { value: "tw", label: "Taiwan" },
    { value: "bili", label: "Bilibili" },
] as const;

export const GRADE_COLORS: Record<string, { bg: string; text: string; border: string }> = {
    S: { bg: "bg-amber-500/20", text: "text-amber-500", border: "border-amber-500/50" },
    A: { bg: "bg-emerald-500/20", text: "text-emerald-500", border: "border-emerald-500/50" },
    B: { bg: "bg-sky-500/20", text: "text-sky-500", border: "border-sky-500/50" },
    C: { bg: "bg-violet-500/20", text: "text-violet-500", border: "border-violet-500/50" },
    D: { bg: "bg-orange-500/20", text: "text-orange-500", border: "border-orange-500/50" },
    F: { bg: "bg-red-500/20", text: "text-red-500", border: "border-red-500/50" },
};

export const SCORE_CATEGORIES = [
    {
        key: "operatorScore",
        label: "Operators",
        description: "Collection size, levels, masteries, modules, potential, and skins",
    },
    { key: "stageScore", label: "Stages", description: "Main story, events, annihilation, and challenge modes" },
    {
        key: "roguelikeScore",
        label: "Roguelike",
        description: "Integrated Strategies endings, collectibles, and achievements",
    },
    { key: "sandboxScore", label: "Sandbox", description: "Reclamation Algorithm progress and unlocks" },
    { key: "medalScore", label: "Medals", description: "Medal collection across all categories" },
    { key: "baseScore", label: "Base", description: "Rhodes Island infrastructure and furniture collection" },
] as const;

/** Default avatar - Amiya E1 */
const DEFAULT_AVATAR = "/api/cdn/avatar/char_002_amiya";

/**
 * Normalizes an avatar ID for use in CDN URLs.
 * Handles these cases:
 * 1. No avatar: Returns default Amiya E1 avatar
 * 2. Custom skins (contains @): Replace @ with _, encode # as %23
 * 3. E2 skins (ends with #2): Replace # with _ → char_xxx_2
 * 4. E0/E1 skins (ends with #1): Strip the #1 suffix → char_xxx (base ID)
 * 5. Base character IDs (no @ or #): Keep as is → char_xxx
 */
export function getAvatarUrl(avatarId: string | null): string {
    if (!avatarId) return DEFAULT_AVATAR;

    // Custom skins use @ and need special handling for #
    if (avatarId.includes("@")) {
        const normalizedId = avatarId.replaceAll("@", "_").replaceAll("#", "%23");
        return `/api/cdn/avatar/${normalizedId}`;
    }

    // E0/E1 skins end with #1 - use base character ID (strip the #1)
    if (avatarId.endsWith("#1")) {
        const baseId = avatarId.slice(0, -2); // Remove "#1"
        return `/api/cdn/avatar/${baseId}`;
    }

    // E2 or other phase skins have # (e.g., char_002_amiya#2)
    if (avatarId.includes("#")) {
        const normalizedId = avatarId.replaceAll("#", "_");
        return `/api/cdn/avatar/${normalizedId}`;
    }

    // Base character ID without phase suffix - keep as is
    return `/api/cdn/avatar/${avatarId}`;
}
