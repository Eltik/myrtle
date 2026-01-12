import { Medal } from "lucide-react";
import { cn } from "~/lib/utils";

interface RankBadgeProps {
    rank: number;
}

const RANK_STYLES = {
    1: {
        bg: "bg-amber-500/20",
        iconColor: "text-amber-400",
        glow: "shadow-[0_0_12px_rgba(251,191,36,0.5)]",
    },
    2: {
        bg: "bg-slate-400/20",
        iconColor: "text-slate-300",
        glow: "",
    },
    3: {
        bg: "bg-amber-700/20",
        iconColor: "text-amber-600",
        glow: "",
    },
};

export function RankBadge({ rank }: RankBadgeProps) {
    if (rank >= 1 && rank <= 3) {
        const style = RANK_STYLES[rank as 1 | 2 | 3];

        return (
            <div className="flex items-center justify-center">
                <div className={cn("flex h-8 w-8 items-center justify-center rounded-full", style.bg, style.glow)}>
                    <Medal className={cn("h-4 w-4", style.iconColor)} />
                </div>
            </div>
        );
    }

    return <span className={cn("font-mono text-muted-foreground", rank <= 10 && "font-medium text-foreground")}>{rank}</span>;
}
