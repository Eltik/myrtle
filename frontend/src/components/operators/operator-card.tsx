"use client";

import Link from "next/link";
import Image from "next/image";
import { cn } from "~/lib/utils";

interface Operator {
    id: string;
    name: string;
    rarity: number;
    class: string;
    subclass: string;
    element: string;
    image: string;
}

interface OperatorCardProps {
    operator: Operator;
    viewMode: "grid" | "list";
}

const RARITY_COLORS: Record<number, { border: string; bg: string; glow: string }> = {
    6: {
        border: "border-amber-500/60",
        bg: "from-amber-500/20 via-orange-500/10 to-transparent",
        glow: "group-hover:shadow-[0_0_20px_rgba(245,158,11,0.3)]",
    },
    5: {
        border: "border-yellow-400/50",
        bg: "from-yellow-400/15 via-yellow-500/5 to-transparent",
        glow: "group-hover:shadow-[0_0_15px_rgba(250,204,21,0.25)]",
    },
    4: {
        border: "border-purple-400/40",
        bg: "from-purple-400/10 via-purple-500/5 to-transparent",
        glow: "group-hover:shadow-[0_0_12px_rgba(192,132,252,0.2)]",
    },
    3: {
        border: "border-blue-400/30",
        bg: "from-blue-400/10 via-blue-500/5 to-transparent",
        glow: "group-hover:shadow-[0_0_10px_rgba(96,165,250,0.15)]",
    },
    2: {
        border: "border-green-400/25",
        bg: "from-green-400/8 via-green-500/4 to-transparent",
        glow: "",
    },
    1: {
        border: "border-gray-400/20",
        bg: "from-gray-400/5 to-transparent",
        glow: "",
    },
};

const CLASS_ICONS: Record<string, string> = {
    Guard: "⚔️",
    Sniper: "🎯",
    Defender: "🛡️",
    Medic: "💚",
    Supporter: "🔮",
    Caster: "✨",
    Specialist: "🗡️",
    Vanguard: "⚡",
};

export function OperatorCard({ operator, viewMode }: OperatorCardProps) {
    const rarityStyle = RARITY_COLORS[operator.rarity] ?? RARITY_COLORS[1];

    if (viewMode === "list") {
        return (
            <Link href={`/operators/${operator.id}`} className={cn("group flex items-center gap-4 rounded-lg border bg-card p-3 transition-all duration-200", rarityStyle?.border, "hover:bg-card/80 hover:scale-[1.01]", rarityStyle?.glow)}>
                <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-lg">
                    <Image src={operator.image || "/placeholder.svg"} alt={operator.name} fill className="object-cover" />
                </div>
                <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                        <span className="font-semibold text-foreground truncate">{operator.name}</span>
                        <span className="text-amber-400 text-sm">{"★".repeat(operator.rarity)}</span>
                    </div>
                    <div className="flex items-center gap-2 text-muted-foreground text-sm">
                        <span>{CLASS_ICONS[operator.class]}</span>
                        <span>{operator.class}</span>
                        <span className="text-border">•</span>
                        <span>{operator.subclass}</span>
                    </div>
                </div>
                <div className={cn("shrink-0 rounded-full px-2 py-0.5 text-xs font-medium", operator.element === "Arts" && "bg-purple-500/20 text-purple-400", operator.element === "Physical" && "bg-orange-500/20 text-orange-400", operator.element === "Healing" && "bg-green-500/20 text-green-400")}>
                    {operator.element}
                </div>
            </Link>
        );
    }

    return (
        <Link href={`/operators/${operator.id}`} className={cn("group relative flex flex-col overflow-hidden rounded-lg border bg-card transition-all duration-200", rarityStyle?.border, "hover:scale-[1.03] hover:-translate-y-1", rarityStyle?.glow)}>
            {/* Rarity gradient overlay */}
            <div className={cn("pointer-events-none absolute inset-0 bg-gradient-to-t", rarityStyle?.bg)} />

            {/* Class icon badge */}
            <div className="absolute top-2 right-2 z-10 flex h-6 w-6 items-center justify-center rounded-md bg-background/80 text-sm backdrop-blur-sm">{CLASS_ICONS[operator.class]}</div>

            {/* Element badge */}
            <div
                className={cn(
                    "absolute top-2 left-2 z-10 rounded-full px-1.5 py-0.5 text-[10px] font-medium backdrop-blur-sm",
                    operator.element === "Arts" && "bg-purple-500/30 text-purple-300",
                    operator.element === "Physical" && "bg-orange-500/30 text-orange-300",
                    operator.element === "Healing" && "bg-green-500/30 text-green-300",
                )}
            >
                {operator.element}
            </div>

            {/* Image container */}
            <div className="relative aspect-square overflow-hidden">
                <Image src={operator.image || "/placeholder.svg"} alt={operator.name} fill className="object-cover transition-transform duration-300 group-hover:scale-110" />
                {/* Bottom gradient for text readability */}
                <div className="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-card via-card/50 to-transparent" />
            </div>

            {/* Info section */}
            <div className="relative -mt-6 z-10 p-3 pt-0">
                <div className="mb-1 flex items-center gap-1">
                    {Array.from({ length: operator.rarity }).map((_, i) => (
                        <span key={i} className="text-[10px] text-amber-400">
                            ★
                        </span>
                    ))}
                </div>
                <h3 className="font-semibold text-foreground text-sm leading-tight truncate">{operator.name}</h3>
                <p className="mt-0.5 text-muted-foreground text-xs truncate">{operator.subclass}</p>
            </div>
        </Link>
    );
}
