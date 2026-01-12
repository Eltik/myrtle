import { RotateCcw, Search } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";

interface EmptyStateProps {
    hasFilters: boolean;
    onReset: () => void;
}

export function EmptyState({ hasFilters, onReset }: EmptyStateProps) {
    return (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed bg-card/30 py-16">
            <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
                <Search className="h-8 w-8 text-muted-foreground" />
            </div>
            <h3 className="mb-2 font-medium text-lg">No players found</h3>
            <p className="mb-4 max-w-sm text-center text-muted-foreground text-sm">{hasFilters ? "Try adjusting your filters or search terms to find more players." : "Start searching for players by name or UID above."}</p>
            {hasFilters && (
                <Button onClick={onReset} variant="outline">
                    <RotateCcw className="mr-2 h-4 w-4" />
                    Clear Filters
                </Button>
            )}
        </div>
    );
}
