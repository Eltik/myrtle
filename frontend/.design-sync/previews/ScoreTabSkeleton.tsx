import { ScoreTabSkeleton } from "frontend";

// What the Score tab shows while `/user/score` is in flight: the overall grade
// card spanning both columns, then one placeholder per subscore.
export const LoadingGrade = () => (
    <div className="mx-auto max-w-3xl">
        <ScoreTabSkeleton />
    </div>
);
