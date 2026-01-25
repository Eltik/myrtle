import { Ellipsis } from "lucide-react";
import type { NavItem } from "./types";

export const navItems: NavItem[] = [
    { label: "Home", href: "/" },
    {
        label: "Operators",
        href: "#",
        dropdown: [
            { label: "Collection", href: "/operators/list", description: "List of all released operators" },
            { label: "Tier List", href: "/operators/tier-list", description: "Operator rankings by class" },
        ],
    },
    {
        label: "Tools",
        href: "#",
        dropdown: [
            { label: "Recruitment Calculator", href: "/tools/recruitment", description: "Calculate recruitment probabilities" },
            { label: "DPS Charts", href: "/tools/dps", description: "Display DPS to compare multiple operators" },
            { label: "Randomizer", href: "/tools/randomizer", description: "Randomize squads and stages" },
        ],
    },
    {
        label: "Gacha",
        href: "#",
        dropdown: [
            { label: "My History", href: "/my/gacha", description: "View your personal pull history" },
            { label: "Global Statistics", href: "/stats/gacha", description: "Community-wide gacha statistics" },
        ],
    },
    {
        label: "Players",
        href: "#",
        dropdown: [
            { label: "Search Players", href: "/users/search", description: "Find and view player profiles" },
            { label: "Leaderboard", href: "/users/leaderboard", description: "Top players and rankings" },
        ],
    },
    {
        label: <Ellipsis />,
        href: "#",
        dropdown: [
            { label: "Privacy", href: "/privacy", description: "Privacy policy" },
            { label: "Terms", href: "/terms", description: "Terms of service" },
            { label: "GitHub", href: "/github", description: "View source code", external: true },
            { label: "Discord", href: "/discord", description: "Join our community", external: true },
        ],
    },
];
