import { BookOpen, Coins, FileText, Music, Palette, Sparkles } from "lucide-react";
import type React from "react";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio" | "lore";

export const TABS: { type: TabType; label: string; icon: React.ElementType }[] = [
    { type: "info", label: "Information", icon: BookOpen },
    { type: "skills", label: "Skills & Talents", icon: Sparkles },
    { type: "levelup", label: "Level-Up Cost", icon: Coins },
    { type: "skins", label: "Skins", icon: Palette },
    { type: "audio", label: "Audio/SFX", icon: Music },
    { type: "lore", label: "Lore", icon: FileText },
];
