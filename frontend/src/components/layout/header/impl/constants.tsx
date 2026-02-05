import { Ellipsis } from "lucide-react";
import type { NavItem } from "./types";

export const navItems: NavItem[] = [
    { label: "Home", href: "/" },
    {
        label: "Collection",
        href: "#",
        dropdown: [
            { label: "Operators", href: "/collection/operators", description: "List of all released operators" },
            { label: "Enemies", href: "/collection/enemies", description: "Browse all enemy data and stats" },
        ],
    },
    { label: "Tier List", href: "/tier-list" },
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
            { label: "My History", href: "/gacha/history", description: "View your personal pull history" },
            { label: "Community Stats", href: "/gacha/community", description: "Community-wide gacha statistics" },
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
