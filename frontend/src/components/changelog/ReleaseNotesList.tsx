import { Link } from "@tanstack/react-router";
import type React from "react";
import { formatNoteDate, ReleaseNoteItems } from "#/components/changelog/release-note-shared";
import type { messages as sharedMessages } from "#/components/changelog/release-note-shared.messages";
import { Button } from "#/components/ui/button";
import { RELEASE_NOTES } from "#/content/changelog/entries";
import type { messages as entryMessages } from "#/content/changelog/entries.messages";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Markdown } from "#/lib/markdown";
import type { messages } from "./ReleaseNotesList.messages";

/**
 * The archive half of the curated tier. Same entries the dialog draws its
 * marquee from, rendered in full and including the ones filed with
 * `announce: false` that never interrupted anybody.
 */
export function ReleaseNotesList(): React.ReactElement {
    const t: TypedT<typeof messages & typeof sharedMessages & typeof entryMessages> = useT("changelog");
    const locale = useLocale();

    if (RELEASE_NOTES.length === 0) {
        return (
            <div className="flex flex-col items-center rounded-2xl border border-border border-dashed bg-card/40 px-6 py-14 text-center">
                <p className="font-sans text-[13px] text-muted-foreground">{t("list.empty")}</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-5">
            {RELEASE_NOTES.map((note) => (
                <article key={note.id} className="overflow-hidden rounded-2xl border border-border bg-card">
                    {note.media && <img alt={t(note.media.altKey)} className="aspect-video w-full border-border border-b object-cover" src={note.media.src} />}
                    <div className="px-5 py-5 sm:px-6">
                        <span className="font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">{formatNoteDate(note.date, locale)}</span>
                        <h2 className="mt-2 font-heading font-semibold text-foreground text-lg leading-snug">{t(note.titleKey)}</h2>
                        <Markdown className="mt-2 text-muted-foreground text-sm leading-relaxed" flush text={t(note.leadKey)} />

                        {note.items && note.items.length > 0 && (
                            <div className="mt-5 border-border border-t pt-4">
                                <ReleaseNoteItems items={note.items} />
                            </div>
                        )}

                        {note.href && (
                            <Button className="mt-5" render={<Link to={note.href} />} size="sm" variant="outline">
                                {note.hrefLabelKey ? t(note.hrefLabelKey) : t("note.takeALook")}
                            </Button>
                        )}
                    </div>
                </article>
            ))}
        </div>
    );
}
