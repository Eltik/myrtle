import { Activity, Calculator, ChessQueen, Dices, TrendingUp, Users } from "lucide-react";

export const FEATURES = [
    {
        icon: Users,
        title: "Operator Database",
        description: "Browse and analyze all Arknights operators with detailed stats, skills, and artwork.",
        href: "/collection/operators",
    },
    {
        icon: TrendingUp,
        title: "Tier Lists",
        description: "Community-driven tier lists to help you build the strongest team compositions.",
        href: "/tools/tier-list",
    },
    {
        icon: Calculator,
        title: "Recruitment Calculator",
        description: "Find the best tag combinations to recruit your desired operators efficiently.",
        href: "/tools/recruitment",
    },
    {
        icon: ChessQueen,
        title: "Player Leaderboards",
        description: "Check how your account stacks up against other players.",
        href: "/users/leaderboard",
    },
    {
        icon: Activity,
        title: "DPS Charts",
        description: "Graph and display the average damage per second dealt by each operator.",
        href: "/tools/dps",
    },
    {
        icon: Dices,
        title: "Randomizer",
        description: "Create random squads and use them in randomized stages.",
        href: "/tools/randomizer",
    },
] as const;
