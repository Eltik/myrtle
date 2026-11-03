import { Check } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Toggle } from "#/components/ui/toggle";
import { cn } from "#/lib/utils";

interface IFilterChipProps {
    label: React.ReactNode;
    active: boolean;
    count?: number;
    onSelect: () => void;
    /** A CSS colour for the pressed tint and border, e.g. `var(--dmg-magic)`. Primary by default. */
    tone?: string;
    className?: string;
}

/**
 * Toggle chip with an optional count badge, for list filter rows. It keeps the Toggle's
 * radius, type scale, `aria-pressed` and coarse-pointer 44px hit area. The active tint goes
 * through `data-pressed:`: unprefixed, the Toggle's own `data-pressed:bg-input` outranked
 * it, so the chip measured `--input` (light) and foreground/12 (dark) when pressed and the
 * primary tint and its 1px ring never painted. Text stays foreground, since tone text on
 * its own tint fails AA (1.66 to 3.96 in audit-002), and a check mark carries the pressed
 * state without relying on colour.
 */
export function FilterChip({ label, active, count, onSelect, tone, className }: IFilterChipProps) {
    return (
        <Toggle
            className={cn("h-8 gap-1.5 px-3 [--tone:var(--primary)] data-pressed:inset-ring-0 data-pressed:border-(--tone)/50 data-pressed:bg-(--tone)/10 data-pressed:text-foreground data-pressed:hover:bg-(--tone)/15 dark:data-pressed:bg-(--tone)/10 dark:data-pressed:hover:bg-(--tone)/15", className)}
            onPressedChange={() => onSelect()}
            pressed={active}
            size="sm"
            style={tone ? ({ "--tone": tone } as React.CSSProperties) : undefined}
            variant="outline"
        >
            {active && <Check aria-hidden className="-ml-0.5 size-3.5" />}
            <span>{label}</span>
            {count != null && (
                <Badge className="font-mono tabular-nums" size="sm" variant={active ? "default" : "secondary"}>
                    {count}
                </Badge>
            )}
        </Toggle>
    );
}
