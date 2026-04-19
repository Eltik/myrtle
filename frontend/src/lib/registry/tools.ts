export type ToolIconName = "chart" | "calc" | "star" | "dice" | "pack";

export interface Tool {
    id: string;
    href: string;
    label: string;
    desc: string;
    icon: ToolIconName;
    shortcut?: string;
    kb?: string[];
    keywords: string[];
}

export const TOOLS: Tool[] = [
    {
        id: "dps",
        href: "/dps",
        label: "DPS charts",
        desc: "Interactive damage curves per skill",
        icon: "chart",
        shortcut: "⌘D",
        kb: ["⌘", "D"],
        keywords: ["damage", "dps", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "recruitment",
        href: "/recruitment",
        label: "Recruitment calculator",
        desc: "Guaranteed tag combos · 1h parity",
        icon: "calc",
        shortcut: "⌘R",
        kb: ["⌘", "R"],
        keywords: ["recruit", "tag", "calculator", "hire"],
    },
    {
        id: "tier-lists",
        href: "/tier-lists",
        label: "Tier lists",
        desc: "Community ranks · live voting",
        icon: "star",
        shortcut: "⌘T",
        kb: ["⌘", "T"],
        keywords: ["rank", "tier", "list", "meta", "community"],
    },
    {
        id: "randomizer",
        href: "/randomizer",
        label: "Randomizer",
        desc: "Pick a squad, break the meta",
        icon: "dice",
        shortcut: "⌘Z",
        kb: ["⌘", "Z"],
        keywords: ["random", "squad", "pick", "roll"],
    },
];
