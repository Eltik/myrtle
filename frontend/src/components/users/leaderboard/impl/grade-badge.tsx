import { cn } from "~/lib/utils";
import { GRADE_COLORS } from "./constants";

interface GradeBadgeProps {
    grade: string;
    size?: "sm" | "md" | "lg";
    showLabel?: boolean;
}

export function GradeBadge({ grade, size = "md", showLabel = false }: GradeBadgeProps) {
    const colors = GRADE_COLORS[grade] || GRADE_COLORS.F;

    const sizeClasses = {
        sm: "h-5 w-5 text-xs",
        md: "h-7 w-7 text-sm",
        lg: "h-9 w-9 text-base",
    };

    return (
        <div className="flex items-center justify-center gap-2">
            <div className={cn("flex items-center justify-center rounded-md border font-bold", colors.bg, colors.text, colors.border, sizeClasses[size])}>{grade}</div>
            {showLabel && <span className="text-muted-foreground text-sm">Grade</span>}
        </div>
    );
}
