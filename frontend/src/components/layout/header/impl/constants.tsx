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
            { label: "Chibi Viewer", href: "/operators/chibis", description: "Arknights chibi viewer" },
        ],
    },
    {
        label: "Tools",
        href: "#",
        dropdown: [
            { label: "Recruitment Calculator", href: "/tools/recruitment", description: "Calculate recruitment probabilities" },
            { label: "DPS Charts", href: "/tools/dps", description: "Display DPS to compare multiple operators" },
            { label: "Squad Randomizer", href: "/tools/squad", description: "Randomize a 12-operator squad" },
        ],
    },
    {
        label: "Events",
        href: "#",
        dropdown: [
            { label: "Current Events", href: "/events/current", description: "Active event guides" },
            { label: "Event Archive", href: "/events/archive", description: "Past event information" },
            { label: "CN Preview", href: "/events/cn-preview", description: "Upcoming content from CN" },
        ],
    },
    {
        label: <Ellipsis />,
        href: "#",
        dropdown: [
            { label: "About", href: "/about", description: "About this project" },
            { label: "Changelog", href: "/changelog", description: "Recent updates" },
            { label: "Terms", href: "/terms", description: "Terms of use" },
        ],
    },
];
