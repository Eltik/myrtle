export type ToolIconName = "chart" | "calc" | "star" | "dice" | "pack" | "search" | "trophy" | "users" | "history";

export interface ITool {
    id: string;
    href: string;
    label: string;
    desc: string;
    icon: ToolIconName;
    key?: string;
    keywords: string[];
}

export function modKey(isMac: boolean): string {
    return isMac ? "⌘" : "CTRL";
}

export function toolKb(tool: ITool, isMac: boolean): string[] | undefined {
    return tool.key ? [modKey(isMac), tool.key] : undefined;
}

export function toolShortcut(tool: ITool, isMac: boolean): string | undefined {
    if (!tool.key) return undefined;
    return isMac ? `⌘${tool.key}` : `CTRL+${tool.key}`;
}

export const TOOLS: ITool[] = [
    {
        id: "dps",
        href: "/tools/dps",
        label: "DPS charts",
        desc: "Interactive damage curves per skill",
        icon: "chart",
        key: "D",
        keywords: ["damage", "dps", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "recruitment",
        href: "/tools/recruitment",
        label: "Recruitment calculator",
        desc: "Guaranteed tag combos · 1h parity",
        icon: "calc",
        key: "R",
        keywords: ["recruit", "tag", "calculator", "hire"],
    },
    {
        id: "randomizer",
        href: "/tools/randomizer",
        label: "Randomizer",
        desc: "Pick a squad, break the meta",
        icon: "dice",
        key: "Z",
        keywords: ["random", "squad", "pick", "roll"],
    },
];
