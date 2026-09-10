import type React from "react";
import type { IReleaseNoteItem, ReleaseNoteKind } from "#/content/changelog/entries";
import { cn } from "#/lib/utils";

export const KIND_LABEL: Record<ReleaseNoteKind, string> = { new: "New", improved: "Improved", fixed: "Fixed" };

/**
 * A dot plus a label, never a filled badge. Three coloured pills stacked in a
 * short list reads as a rainbow and flattens the hierarchy the marquee sets up.
 * `fixed` is deliberately the quietest: nobody opens a dialog to read bug fixes.
 */
export const KIND_DOT: Record<ReleaseNoteKind, string> = {
    new: "bg-primary",
    improved: "bg-chart-2",
    fixed: "bg-muted-foreground/60",
};

/** Parses at LOCAL midnight. `new Date("2026-09-10")` is UTC and renders as the 9th west of Greenwich. */
export function formatNoteDate(iso: string): string {
    const parsed = new Date(`${iso}T00:00:00`);
    if (Number.isNaN(parsed.getTime())) return iso;
    return parsed.toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });
}

export function ReleaseNoteItems({ items }: { items: IReleaseNoteItem[] }): React.ReactElement {
    return (
        <ul className="flex flex-col gap-2.5">
            {items.map((item) => (
                <li key={`${item.kind}-${item.text}`} className="flex gap-2.5 text-sm leading-relaxed">
                    <span aria-hidden="true" className={cn("mt-[0.45rem] size-1.5 shrink-0 rounded-full", KIND_DOT[item.kind])} />
                    <span className="min-w-0">
                        <span className="mr-1.5 font-medium text-foreground">{KIND_LABEL[item.kind]}</span>
                        <span className="text-muted-foreground">{item.text}</span>
                    </span>
                </li>
            ))}
        </ul>
    );
}
