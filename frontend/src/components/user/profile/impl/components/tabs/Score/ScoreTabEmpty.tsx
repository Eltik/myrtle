import { Trophy } from "lucide-react";

export function ScoreTabEmpty() {
    return (
        <div className="flex flex-col items-center justify-center gap-3 rounded-2xl border border-border bg-card px-8 py-16 text-center">
            <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-muted/40">
                <Trophy className="h-5 w-5 text-muted-foreground/60" />
            </div>
            <h2 className="font-semibold text-lg tracking-tight">No score yet</h2>
            <p className="max-w-sm text-muted-foreground text-sm">This Doctor's grade hasn't been calculated. Scores are computed periodically once enough data is on file.</p>
        </div>
    );
}
