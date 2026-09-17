import type { messages as toolMessages } from "#/lib/registry/tools.messages";

export type ToolIconName = "chart" | "calc" | "star" | "dice" | "cake" | "pack" | "search" | "trophy" | "users" | "user" | "history" | "tiers" | "heart" | "list-todo" | "shield" | "crosshair" | "map" | "calendar-clock";

export type ToolCategory = "calculator" | "fun";

/**
 * A key in `tools.messages.ts`. This module has no React, so it cannot resolve
 * text itself; it carries keys and the rendering component calls `t()`.
 */
export type ToolMessageKey = keyof typeof toolMessages & string;

export interface ITool {
    id: string;
    href: string;
    labelKey: ToolMessageKey;
    descKey: ToolMessageKey;
    icon: ToolIconName;
    category: ToolCategory;
    keywords: string[];
}

export interface IToolCategory {
    id: ToolCategory;
    labelKey: ToolMessageKey;
}

export const TOOL_CATEGORIES: IToolCategory[] = [
    { id: "calculator", labelKey: "toolCategory.calculator" },
    { id: "fun", labelKey: "toolCategory.fun" },
];

export function modKey(isMac: boolean): string {
    return isMac ? "⌘" : "Ctrl";
}

export const TOOLS: ITool[] = [
    {
        id: "recruitment",
        href: "/tools/recruitment",
        labelKey: "tool.recruitment.label",
        descKey: "tool.recruitment.desc",
        icon: "calc",
        category: "calculator",
        keywords: ["recruit", "tag", "calculator", "hire"],
    },
    {
        id: "planner",
        href: "/tools/planner",
        labelKey: "tool.planner.label",
        descKey: "tool.planner.desc",
        icon: "list-todo",
        category: "calculator",
        keywords: ["plan", "planner", "promotion", "skill", "module", "target"],
    },
    {
        id: "dps",
        href: "/tools/dps",
        labelKey: "tool.dps.label",
        descKey: "tool.dps.desc",
        icon: "chart",
        category: "calculator",
        keywords: ["damage", "dps", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "hps",
        href: "/tools/hps",
        labelKey: "tool.hps.label",
        descKey: "tool.hps.desc",
        icon: "heart",
        category: "calculator",
        keywords: ["healing", "hps", "heal", "medic", "chart", "skill", "curve", "calculator"],
    },
    {
        id: "randomizer",
        href: "/tools/randomizer",
        labelKey: "tool.randomizer.label",
        descKey: "tool.randomizer.desc",
        icon: "dice",
        category: "fun",
        keywords: ["random", "squad", "pick", "roll"],
    },
    {
        id: "birthdays",
        href: "/tools/birthdays",
        labelKey: "tool.birthdays.label",
        descKey: "tool.birthdays.desc",
        icon: "cake",
        category: "fun",
        keywords: ["birthday", "operator", "track"],
    },
    {
        id: "release",
        href: "/tools/release",
        labelKey: "tool.release.label",
        descKey: "tool.release.desc",
        icon: "calendar-clock",
        category: "calculator",
        keywords: ["release", "upcoming", "events", "banners", "skins", "cn", "en", "schedule", "planner", "lag", "originite", "prime", "op", "budget", "pulls", "orundum", "headhunting", "gacha", "pity", "rates", "odds"],
    },
];

export function getToolsByCategory(): { category: IToolCategory; tools: ITool[] }[] {
    return TOOL_CATEGORIES.map((category) => ({
        category,
        tools: TOOLS.filter((tool) => tool.category === category.id),
    })).filter((group) => group.tools.length > 0);
}
