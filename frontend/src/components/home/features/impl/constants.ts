import type { LucideIcon } from "lucide-react";
import { Calculator, Clock, Database, Shield, Sparkles, Users } from "lucide-react";

export interface Feature {
    icon: LucideIcon;
    title: string;
    description: string;
    href?: string;
}

export const FEATURES: Feature[] = [
    {
        icon: Database,
        title: "Operator Database",
        description: "Browse detailed stats, skills, and builds for every operator in Arknights.",
        href: "/operators/list",
    },
    {
        icon: Calculator,
        title: "Recruitment Calculator",
        description: "Find the perfect operator combinations with our advanced tag calculator.",
        href: "/tools/recruitment",
    },
    {
        icon: Sparkles,
        title: "Tier Lists",
        description: "Compare operators with community-driven tier lists and rankings.",
        href: "/operators/tier-list",
    },
    {
        icon: Clock,
        title: "Event Timeline",
        description: "Stay updated on current and upcoming events, banners, and schedules.",
    },
    {
        icon: Shield,
        title: "Team Builder",
        description: "Create and optimize squad compositions for any stage or challenge.",
    },
    {
        icon: Users,
        title: "Community Hub",
        description: "Connect with other Doctors, share strategies, and join discussions.",
    },
] as const;

export const ANIMATION_VARIANTS = {
    card: {
        hidden: { opacity: 0, y: 30, scale: 0.95 },
        visible: { opacity: 1, y: 0, scale: 1 },
    },
    header: {
        hidden: { opacity: 0, y: 30 },
        visible: { opacity: 1, y: 0 },
    },
} as const;

export const getCardDelay = (index: number): number => 0.1 + index * 0.1;
