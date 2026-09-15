import type React from "react";
import type { IReleaseNoteItem, ReleaseNoteKind } from "#/content/changelog/entries";
import type { messages as entryMessages } from "#/content/changelog/entries.messages";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./release-note-shared.messages";

/** This file's own keys plus the entry prose the notes point at. */
export type ReleaseNoteT = TypedT<typeof messages & typeof entryMessages>;

export const KIND_LABEL_KEYS: Record<ReleaseNoteKind, keyof typeof messages & string> = { new: "note.kind.new", improved: "note.kind.improved", fixed: "note.kind.fixed" };

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
export function formatNoteDate(iso: string, locale?: string): string {
    const parsed = new Date(`${iso}T00:00:00`);
    if (Number.isNaN(parsed.getTime())) return iso;
    return parsed.toLocaleDateString(locale, { year: "numeric", month: "long", day: "numeric" });
}

export function ReleaseNoteItems({ items }: { items: IReleaseNoteItem[] }): React.ReactElement {
    const t: ReleaseNoteT = useT("changelog");
    return (
        <ul className="flex flex-col gap-2.5">
            {items.map((item) => (
                <li key={`${item.kind}-${item.textKey}`} className="flex gap-2.5 text-sm leading-relaxed">
                    <span aria-hidden="true" className={cn("mt-[0.45rem] size-1.5 shrink-0 rounded-full", KIND_DOT[item.kind])} />
                    <span className="min-w-0">
                        <span className="mr-1.5 font-medium text-foreground">{t(KIND_LABEL_KEYS[item.kind])}</span>
                        <span className="text-muted-foreground">{t(item.textKey)}</span>
                    </span>
                </li>
            ))}
        </ul>
    );
}
