import { Building2, Compass, type LucideIcon, Map as MapIcon, Medal, Swords, TreePalm } from "lucide-react";

// Score category configuration with icons and colors
export const SCORE_CATEGORY_CONFIG: Record<
    string,
    {
        key: string;
        label: string;
        description: string;
        icon: LucideIcon;
        color: string;
        bgColor: string;
        progressColor: string;
    }
> = {
    operatorScore: {
        key: "operatorScore",
        label: "Operators",
        description: "Collection, levels, masteries, modules",
        icon: Swords,
        color: "text-rose-400",
        bgColor: "bg-rose-500/10",
        progressColor: "bg-rose-500",
    },
    stageScore: {
        key: "stageScore",
        label: "Stages",
        description: "Main story, events, challenges",
        icon: MapIcon,
        color: "text-sky-400",
        bgColor: "bg-sky-500/10",
        progressColor: "bg-sky-500",
    },
    roguelikeScore: {
        key: "roguelikeScore",
        label: "Roguelike",
        description: "Integrated Strategies progress",
        icon: Compass,
        color: "text-violet-400",
        bgColor: "bg-violet-500/10",
        progressColor: "bg-violet-500",
    },
    sandboxScore: {
        key: "sandboxScore",
        label: "Sandbox",
        description: "Reclamation Algorithm unlocks",
        icon: TreePalm,
        color: "text-emerald-400",
        bgColor: "bg-emerald-500/10",
        progressColor: "bg-emerald-500",
    },
    medalScore: {
        key: "medalScore",
        label: "Medals",
        description: "Medal collection progress",
        icon: Medal,
        color: "text-amber-400",
        bgColor: "bg-amber-500/10",
        progressColor: "bg-amber-500",
    },
    baseScore: {
        key: "baseScore",
        label: "Base",
        description: "Infrastructure & furniture",
        icon: Building2,
        color: "text-cyan-400",
        bgColor: "bg-cyan-500/10",
        progressColor: "bg-cyan-500",
    },
};

// Grade color configuration with extended styling
export const GRADE_CONFIG: Record<
    string,
    {
        bg: string;
        text: string;
        border: string;
        glow: string[];
        gradient: string;
        ringColor: string;
    }
> = {
    S: {
        bg: "bg-rose-500/10",
        text: "text-rose-400",
        border: "border-rose-500/30",
        glow: ["#fb7185", "#f43f5e", "#e11d48"],
        gradient: "from-rose-500/20 to-rose-500/5",
        ringColor: "stroke-rose-500",
    },
    A: {
        bg: "bg-orange-500/10",
        text: "text-orange-400",
        border: "border-orange-500/30",
        glow: ["#fb923c", "#f97316", "#ea580c"],
        gradient: "from-orange-500/20 to-orange-500/5",
        ringColor: "stroke-orange-500",
    },
    B: {
        bg: "bg-amber-500/10",
        text: "text-amber-400",
        border: "border-amber-500/30",
        glow: ["#fbbf24", "#f59e0b", "#d97706"],
        gradient: "from-amber-500/20 to-amber-500/5",
        ringColor: "stroke-amber-500",
    },
    C: {
        bg: "bg-yellow-500/10",
        text: "text-yellow-400",
        border: "border-yellow-500/30",
        glow: ["#facc15", "#eab308", "#ca8a04"],
        gradient: "from-yellow-500/20 to-yellow-500/5",
        ringColor: "stroke-yellow-500",
    },
    D: {
        bg: "bg-lime-500/10",
        text: "text-lime-400",
        border: "border-lime-500/30",
        glow: ["#a3e635", "#84cc16", "#65a30d"],
        gradient: "from-lime-500/20 to-lime-500/5",
        ringColor: "stroke-lime-500",
    },
    F: {
        bg: "bg-slate-500/10",
        text: "text-slate-400",
        border: "border-slate-500/30",
        glow: ["#94a3b8", "#64748b", "#475569"],
        gradient: "from-slate-500/20 to-slate-500/5",
        ringColor: "stroke-slate-500",
    },
};

// Activity metrics labels
export const ACTIVITY_METRICS = [
    { key: "loginRecencyScore", label: "Login Recency", max: 100 },
    { key: "loginFrequencyScore", label: "Login Frequency", max: 100 },
    { key: "consistencyScore", label: "Consistency", max: 100 },
] as const;

// Engagement metrics labels
export const ENGAGEMENT_METRICS = [
    { key: "contentVarietyScore", label: "Content Variety", max: 100 },
    { key: "roguelikeDepthScore", label: "Roguelike Depth", max: 100 },
    { key: "stageDiversityScore", label: "Stage Diversity", max: 100 },
    { key: "progressionDepthScore", label: "Progression Depth", max: 100 },
] as const;
