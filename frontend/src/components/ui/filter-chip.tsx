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
    /** A CSS colour for the pressed tint, e.g. `var(--dmg-magic)`. Primary by default. */
    tone?: string;
    /**
     * The pressed border, an `-ink` token for the tone (e.g. `DAMAGE_TOKENS[d].ink`). It has
     * to clear 3:1 on the page, which raw data tones often miss; without it the border is
     * the tone at full strength, and with no tone at all it is `--ring` (4.59 light, 8.49 dark).
     */
    toneInk?: string;
    disabled?: boolean;
    /** Overrides the visible label as the accessible name, e.g. to spell out the count. */
    "aria-label"?: string;
    className?: string;
}

type FilterChipProps = IFilterChipProps & Omit<React.ComponentProps<typeof Toggle>, keyof IFilterChipProps | "pressed" | "onPressedChange" | "children" | "size" | "variant">;

/**
 * Toggle chip with an optional count badge, for list filter rows. It keeps the Toggle's
 * radius, type scale, `aria-pressed` and coarse-pointer 44px hit area. The active tint goes
 * through `data-pressed:` so it replaces the Toggle's own pressed fill and inset ring: an
 * untoned chip is tinted `--primary` like a pressed Toggle, a toned chip takes its data
 * colour. The pressed border is solid: `--tone` at /50
 * measured 2.28 light / 2.79 dark on the page, under the 3:1 an edge needs. Text stays foreground, since tone text on
 * its own tint fails AA (1.66 to 3.96 in audit-002), and a check mark carries the pressed
 * state without relying on colour. Other Toggle props pass through, so a chip can be a
 * `TooltipTrigger` render target (an icon-only chip names itself in a tooltip).
 */
export function FilterChip({ label, active, count, onSelect, tone, toneInk, disabled, "aria-label": ariaLabel, className, ...rest }: FilterChipProps) {
    return (
        <Toggle
            {...rest}
            aria-label={ariaLabel}
            className={cn(
                "h-8 gap-1.5 px-3 [--tone-edge:var(--ring)] [--tone:var(--primary)] data-pressed:inset-ring-0 data-pressed:border-(--tone-edge) data-pressed:bg-(--tone)/10 data-pressed:text-foreground data-pressed:hover:bg-(--tone)/15 dark:data-pressed:bg-(--tone)/10 dark:data-pressed:hover:bg-(--tone)/15",
                className,
            )}
            disabled={disabled}
            onPressedChange={() => onSelect()}
            pressed={active}
            size="sm"
            style={tone || toneInk ? ({ "--tone": tone, "--tone-edge": toneInk ?? tone } as React.CSSProperties) : undefined}
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
