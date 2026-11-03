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

/** The zone prefix of a code range: "1-1 ~ 1-12" -> "1", "GT-1 ~ GT-6" -> "GT". */
export function zoneCodeOf(codeRange: string): string | null {
    const first = codeRange.split(" ~ ")[0]?.trim() ?? "";
    const cut = first.lastIndexOf("-");
    return (cut > 0 ? first.slice(0, cut) : first) || null;
}

/**
 * Stand-in for missing stage art: a muted tile with the zone code. Rendered *under*
 * `StagePreview`, so it only shows when there is no image (or the image 404s and hides itself).
 */
export function PreviewFallback({ label, labelClassName = "text-[11px]" }: { label?: string | null; labelClassName?: string }) {
    return (
        <div aria-hidden="true" className="absolute inset-0 flex items-center justify-center bg-muted">
            {label && <span className={`font-mono font-semibold text-muted-foreground leading-none ${labelClassName}`}>{label}</span>}
        </div>
    );
}
