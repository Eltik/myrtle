export const STEPS = [
    {
        number: "01",
        title: "Connect Your Account",
        description: "Link your Arknights profile to automatically sync your roster, materials, and progress.",
        action: {
            label: "Connect Now",
        },
    },
    {
        number: "02",
        title: "Explore & Discover",
        description: "Browse the complete operator database, view detailed stats, and explore tier lists curated by the community.",
        action: {
            label: "Browse Operators",
            href: "/operators/list",
        },
    },
    {
        number: "03",
        title: "Plan & Optimize",
        description: "Use powerful tools like the recruitment calculator and material planner to optimize your gameplay strategy.",
        action: {
            label: "Open Tools",
            href: "/tools/recruitment",
        },
    },
    {
        number: "04",
        title: "Track Progress",
        description: "Monitor your collection, track upgrade progress, and never miss an operator or material you need.",
        action: {
            label: "View Profile",
            href: "/my/profile",
        },
    },
] as const;
