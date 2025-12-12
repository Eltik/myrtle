import type { LucideIcon } from "lucide-react";
import { cn } from "~/lib/utils";

interface StatCardProps {
    label: string;
    value: number | string;
    icon?: LucideIcon;
    className?: string;
}

export function StatCard({ label, value, icon: Icon, className }: StatCardProps) {
    return (
        <div className={cn("flex items-center justify-between rounded-lg border border-border/50 bg-secondary/20 px-4 py-3", className)}>
            <div className="flex items-center gap-2 text-muted-foreground">
                {Icon && <Icon className="h-4 w-4" />}
                <span className="text-sm">{label}</span>
            </div>
            <span className="font-mono font-semibold text-foreground">{value}</span>
        </div>
    );
}
