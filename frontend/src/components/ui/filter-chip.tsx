import { Badge } from "#/components/ui/badge";
import { Toggle } from "#/components/ui/toggle";
import { cn } from "#/lib/utils";

interface IFilterChipProps {
    label: string;
    active: boolean;
    count?: number;
    onSelect: () => void;
    className?: string;
}

/** Rounded toggle chip with an optional count badge, for list filter rows. */
export function FilterChip({ label, active, count, onSelect, className }: IFilterChipProps) {
    return (
        <Toggle
            className={cn("h-8 gap-2 rounded-full px-3 font-medium text-[13px]", active && "border-primary/50 bg-primary/10 text-foreground shadow-[0_0_0_1px_color-mix(in_srgb,var(--primary)_25%,transparent),0_0_12px_color-mix(in_srgb,var(--primary)_20%,transparent)] hover:bg-primary/15", className)}
            onPressedChange={() => onSelect()}
            pressed={active}
            size="sm"
            variant="outline"
        >
            <span>{label}</span>
            {count != null && (
                <Badge className="font-mono tabular-nums" size="sm" variant={active ? "default" : "secondary"}>
                    {count}
                </Badge>
            )}
        </Toggle>
    );
}
