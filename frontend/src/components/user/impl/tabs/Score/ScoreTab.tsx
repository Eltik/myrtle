import type { IUserScore } from "#/lib/api/user";
import { OverallGradeCard } from "./cards/OverallGradeCard";
import { SubscoreCard } from "./cards/SubscoreCard";
import { SUBSCORES } from "./helpers";
import { ScoreTabEmpty } from "./ScoreTabEmpty";
import { ScoreTabSkeleton } from "./ScoreTabSkeleton";

interface IScoreTabProps {
    score: IUserScore | null | undefined;
    isLoading: boolean;
}

export function ScoreTab({ score, isLoading }: IScoreTabProps) {
    if (isLoading && !score) return <ScoreTabSkeleton />;
    if (!score) return <ScoreTabEmpty />;

    return (
        <div className="flex flex-col gap-3 pb-8">
            <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/30 px-3 py-2">
                <span className="rounded-md border border-border/60 bg-background px-1.5 py-0.5 font-mono text-[9.5px] font-semibold uppercase tracking-wider text-muted-foreground">Beta</span>
                <span className="text-[12px] text-muted-foreground">Scoring is subject to change.</span>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
                <OverallGradeCard score={score} />
                {SUBSCORES.map((sub) => (
                    <SubscoreCard key={sub.key} score={score[sub.key]} sub={sub} />
                ))}
            </div>
        </div>
    );
}
