import type { IOperatorListItem } from "#/types/operators";
import { BookOpen, Info, Shirt, Sparkles, TrendingUp, Volume2 } from "lucide-react";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio" | "lore";

export const TABS: { type: TabType; label: string; icon: React.ElementType }[] = [
    { type: "info", label: "Information", icon: Info },
    { type: "skills", label: "Skills & Talents", icon: Sparkles },
    { type: "levelup", label: "Level-Up Cost", icon: TrendingUp },
    { type: "skins", label: "Skins", icon: Shirt },
    { type: "audio", label: "Audio/SFX", icon: Volume2 },
    { type: "lore", label: "Lore", icon: BookOpen },
];

interface IOperatorTabsProps {
    operator: IOperatorListItem;
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

export function OperatorTabs({ operator, activeTab, onTabChange }: IOperatorTabsProps) {
    return <p>Hello world</p>
}
