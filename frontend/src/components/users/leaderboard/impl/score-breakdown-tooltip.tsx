import { HelpCircle } from "lucide-react";
import type React from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { SCORE_CATEGORIES } from "./constants";

interface ScoreBreakdownTooltipProps {
    children: React.ReactNode;
}

export function ScoreBreakdownTooltip({ children }: ScoreBreakdownTooltipProps) {
    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <span className="inline-flex items-center gap-1">
                    {children}
                    <HelpCircle className="h-3.5 w-3.5 text-muted-foreground" />
                </span>
            </TooltipTrigger>
            <TooltipContent className="max-w-xs" side="top" variant="dark">
                <div className="space-y-2">
                    <p className="font-medium">Score Breakdown</p>
                    <p className="text-muted-foreground text-xs">Total score is calculated from multiple categories:</p>
                    <ul className="space-y-1 text-xs">
                        {SCORE_CATEGORIES.map((cat) => (
                            <li key={cat.key}>
                                <span className="font-medium">{cat.label}:</span> <span className="text-muted-foreground">{cat.description}</span>
                            </li>
                        ))}
                    </ul>
                </div>
            </TooltipContent>
        </Tooltip>
    );
}
