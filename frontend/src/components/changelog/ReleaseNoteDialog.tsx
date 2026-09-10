"use client";

import { Link } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import type React from "react";
import { useEffect } from "react";
import { formatNoteDate, ReleaseNoteItems } from "#/components/changelog/release-note-shared";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import type { IReleaseNote } from "#/content/changelog/entries";
import { LATEST_ANNOUNCED_NOTE } from "#/content/changelog/entries";
import { initReleaseNotes, releaseNoteStore, setReleaseNoteDialogOpen } from "#/lib/changelog/store";
import { Markdown } from "#/lib/markdown";

export function ReleaseNoteDialog(): React.ReactElement | null {
    // The store must be initialised here and not only in the bell: SiteChrome
    // drops the Header on /admin, so the button is not always mounted.
    useEffect(() => {
        initReleaseNotes();
    }, []);

    const open = useSelector(releaseNoteStore, (s) => s.dialogOpen);
    const missed = useSelector(releaseNoteStore, (s) => s.missed);
    const note = LATEST_ANNOUNCED_NOTE;

    if (note === null) return null;

    return (
        <Dialog open={open} onOpenChange={setReleaseNoteDialogOpen}>
            <DialogPopup className="sm:max-w-lg" closeProps={note.media ? { className: "absolute end-2 top-2 bg-background/70 backdrop-blur-sm" } : undefined}>
                {note.media && <img src={note.media.src} alt={note.media.alt} className="aspect-video w-full shrink-0 rounded-t-2xl border-b object-cover max-sm:rounded-none" />}

                <DialogHeader>
                    <span className="font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">{formatNoteDate(note.date)}</span>
                    <DialogTitle>{note.title}</DialogTitle>
                    <DialogDescription render={<div />}>
                        <Markdown text={note.lead} className="text-sm leading-relaxed" flush />
                    </DialogDescription>
                </DialogHeader>

                {note.items && note.items.length > 0 && (
                    <DialogPanel>
                        <p className="mb-3 font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">Also in this update</p>
                        <ReleaseNoteItems items={note.items} />
                    </DialogPanel>
                )}

                <DialogFooter className="sm:justify-between">
                    <Link to="/changelog" onClick={() => setReleaseNoteDialogOpen(false)} className="self-center text-muted-foreground text-sm no-underline transition-colors hover:text-foreground max-sm:text-center">
                        {missed > 0 ? `All updates (${missed} more you missed)` : "All updates"}
                    </Link>
                    <PrimaryAction note={note} />
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

/**
 * When the entry deep links, the primary action goes TO THE THING. A dialog
 * whose only exit is "Got it" converts an announcement into nothing.
 */
function PrimaryAction({ note }: { note: IReleaseNote }): React.ReactElement {
    if (note.href) {
        return (
            <div className="flex flex-col-reverse gap-2 sm:flex-row">
                <Button variant="ghost" onClick={() => setReleaseNoteDialogOpen(false)}>
                    Got it
                </Button>
                <Button render={<Link to={note.href} />} onClick={() => setReleaseNoteDialogOpen(false)}>
                    {note.hrefLabel ?? "Take a look"}
                </Button>
            </div>
        );
    }
    return <Button onClick={() => setReleaseNoteDialogOpen(false)}>Got it</Button>;
}
