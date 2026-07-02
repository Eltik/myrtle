import { BookOpen, Boxes, CalendarRange, Dices, FlaskConical, type LucideIcon, Package, ScrollText, ShieldHalf, Skull, TreePine } from "lucide-react";
import type { StageGroupKey } from "#/lib/registry/stage-groups";

export const STAGE_GROUP_ICON: Record<StageGroupKey, LucideIcon> = {
    story: BookOpen,
    events: CalendarRange,
    annihilation: Skull,
    is: Dices,
    ra: TreePine,
    sss: ShieldHalf,
    paradox: FlaskConical,
    cc: ScrollText,
    supplies: Package,
    other: Boxes,
};

/**
 * Intentional-looking stand-in for missing stage art: a tone-tinted wash with a
 * faint hatch pattern and the category glyph. Rendered *under* `StagePreview`,
 * so it only shows when there is no image (or the image 404s and hides itself).
 */
export function PreviewFallback({ tone, group, iconClassName = "h-4.5 w-4.5" }: { tone: string; group: StageGroupKey; iconClassName?: string }) {
    const Icon = STAGE_GROUP_ICON[group];
    return (
        <div aria-hidden="true" className="absolute inset-0 flex items-center justify-center" style={{ background: `linear-gradient(135deg, color-mix(in oklch, ${tone} 11%, var(--card)) 0%, color-mix(in oklch, var(--muted) 40%, var(--card)) 75%)` }}>
            <div className="absolute inset-0 opacity-40" style={{ backgroundImage: `repeating-linear-gradient(135deg, color-mix(in oklch, ${tone} 16%, transparent) 0 1px, transparent 1px 8px)` }} />
            <Icon className={iconClassName} style={{ color: `color-mix(in oklch, ${tone} 60%, var(--muted-foreground))`, opacity: 0.75 }} />
        </div>
    );
}
