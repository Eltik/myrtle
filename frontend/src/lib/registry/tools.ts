export type ToolIconName = "chart" | "calc" | "star" | "dice" | "cake" | "pack" | "search" | "trophy" | "users" | "history" | "tiers" | "heart";

export type ToolCategory = "calculator" | "fun";

export interface ITool {
    id: string;
    href: string;
    label: string;
    desc: string;
    icon: ToolIconName;
    category: ToolCategory;
    keywords: string[];
}

export interface IToolCategory {
    id: ToolCategory;
    label: string;
}

export const TOOL_CATEGORIES: IToolCategory[] = [
    { id: "calculator", label: "Calculators" },
    { id: "fun", label: "For Fun" },
];

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
        category: "calculator",
        keywords: ["damage", "dps", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "hps",
        href: "/tools/hps",
        label: "HPS charts",
        desc: "Interactive healing curves per skill",
        icon: "heart",
        category: "calculator",
        keywords: ["healing", "hps", "heal", "medic", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "recruitment",
        href: "/tools/recruitment",
        label: "Recruitment calculator",
        desc: "Guaranteed tag combos · 1h parity",
        icon: "calc",
        category: "calculator",
        keywords: ["recruit", "tag", "calculator", "hire"],
    },
    {
        id: "randomizer",
        href: "/tools/randomizer",
        label: "Randomizer",
        desc: "Pick a squad, break the meta",
        icon: "dice",
        category: "fun",
        keywords: ["random", "squad", "pick", "roll"],
    },
    {
        id: "birthdays",
        href: "/tools/birthdays",
        label: "Birthdays",
        desc: "View and track operator birthdays",
        icon: "cake",
        category: "fun",
        keywords: ["birthday", "operator", "track"],
    },
];

export function getToolsByCategory(): { category: IToolCategory; tools: ITool[] }[] {
    return TOOL_CATEGORIES.map((category) => ({
        category,
        tools: TOOLS.filter((tool) => tool.category === category.id),
    })).filter((group) => group.tools.length > 0);
}
