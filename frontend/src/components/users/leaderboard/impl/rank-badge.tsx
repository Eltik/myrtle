import { Crown, Medal, Trophy } from "lucide-react";
import { cn } from "~/lib/utils";

interface RankBadgeProps {
    rank: number;
}

export function RankBadge({ rank }: RankBadgeProps) {
    if (rank === 1) {
        return (
            <div className="flex items-center justify-center">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-amber-500/20">
                    <Trophy className="h-4 w-4 text-amber-500" />
                </div>
            </div>
        );
    }

    if (rank === 2) {
        return (
            <div className="flex items-center justify-center">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-slate-400/20">
                    <Medal className="h-4 w-4 text-slate-400" />
                </div>
            </div>
        );
    }

    if (rank === 3) {
        return (
            <div className="flex items-center justify-center">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-amber-700/20">
                    <Crown className="h-4 w-4 text-amber-700" />
                </div>
            </div>
        );
    }

    return <span className={cn("font-mono text-muted-foreground", rank <= 10 && "font-medium text-foreground")}>{rank}</span>;
}
