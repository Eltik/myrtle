import { Link } from "@tanstack/react-router";
import type React from "react";
import { formatNoteDate, ReleaseNoteItems } from "#/components/changelog/release-note-shared";
import { Button } from "#/components/ui/button";
import { RELEASE_NOTES } from "#/content/changelog/entries";
import { Markdown } from "#/lib/markdown";

/**
 * The archive half of the curated tier. Same entries the dialog draws its
 * marquee from, rendered in full and including the ones filed with
 * `announce: false` that never interrupted anybody.
 */
export function ReleaseNotesList(): React.ReactElement {
    if (RELEASE_NOTES.length === 0) {
        return (
            <div className="flex flex-col items-center rounded-2xl border border-border border-dashed bg-card/40 px-6 py-14 text-center">
                <p className="font-sans text-[13px] text-muted-foreground">No release notes written yet. The Commits tab has the raw history.</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-5">
            {RELEASE_NOTES.map((note) => (
                <article key={note.id} className="overflow-hidden rounded-2xl border border-border bg-card">
                    {note.media && <img alt={note.media.alt} className="aspect-video w-full border-border border-b object-cover" src={note.media.src} />}
                    <div className="px-5 py-5 sm:px-6">
                        <span className="font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">{formatNoteDate(note.date)}</span>
                        <h2 className="mt-2 font-heading font-semibold text-foreground text-lg leading-snug">{note.title}</h2>
                        <Markdown className="mt-2 text-muted-foreground text-sm leading-relaxed" flush text={note.lead} />

                        {note.items && note.items.length > 0 && (
                            <div className="mt-5 border-border border-t pt-4">
                                <ReleaseNoteItems items={note.items} />
                            </div>
                        )}

                        {note.href && (
                            <Button className="mt-5" render={<Link to={note.href} />} size="sm" variant="outline">
                                {note.hrefLabel ?? "Take a look"}
                            </Button>
                        )}
                    </div>
                </article>
            ))}
        </div>
    );
}
