export type ToolIconName = "chart" | "calc" | "star" | "dice" | "pack" | "search" | "trophy" | "users" | "history" | "tiers";

export interface ITool {
    id: string;
    href: string;
    label: string;
    desc: string;
    icon: ToolIconName;
    keywords: string[];
}

export function modKey(isMac: boolean): string {
    return isMac ? "⌘" : "Ctrl";
}

export const TOOLS: ITool[] = [
    {
        id: "dps",
        href: "/tools/dps",
        label: "DPS charts",
        desc: "Interactive damage curves per skill",
        icon: "chart",
        keywords: ["damage", "dps", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "recruitment",
        href: "/tools/recruitment",
        label: "Recruitment calculator",
        desc: "Guaranteed tag combos · 1h parity",
        icon: "calc",
        keywords: ["recruit", "tag", "calculator", "hire"],
    },
    {
        id: "randomizer",
        href: "/tools/randomizer",
        label: "Randomizer",
        desc: "Pick a squad, break the meta",
        icon: "dice",
        keywords: ["random", "squad", "pick", "roll"],
    },
];
