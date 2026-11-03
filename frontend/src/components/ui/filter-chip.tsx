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

/**
 * Toggle chip with an optional count badge, for list filter rows. It keeps the Toggle's
 * radius and type scale. The active tint goes through `data-pressed:`: unprefixed, the
 * Toggle's own `data-pressed:bg-input` outranked it, so the chip measured `--input`
 * (light) and foreground/12 (dark) when pressed and the primary tint and its 1px ring
 * never painted. Text stays foreground, since primary text on a tint fails AA.
 */
export function FilterChip({ label, active, count, onSelect, className }: IFilterChipProps) {
    return (
        <Toggle
            className={cn("h-8 gap-2 px-3 data-pressed:inset-ring-0 data-pressed:border-primary/50 data-pressed:bg-primary/10 data-pressed:text-foreground data-pressed:hover:bg-primary/15 dark:data-pressed:bg-primary/10 dark:data-pressed:hover:bg-primary/15", className)}
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
