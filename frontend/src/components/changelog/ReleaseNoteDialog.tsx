"use client";

import { Link } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import type React from "react";
import { useEffect } from "react";
import { formatNoteDate, ReleaseNoteItems } from "#/components/changelog/release-note-shared";
import type { messages as sharedMessages } from "#/components/changelog/release-note-shared.messages";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import type { IReleaseNote } from "#/content/changelog/entries";
import { LATEST_ANNOUNCED_NOTE } from "#/content/changelog/entries";
import type { messages as entryMessages } from "#/content/changelog/entries.messages";
import { initReleaseNotes, releaseNoteStore, setReleaseNoteDialogOpen } from "#/lib/changelog/store";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Markdown } from "#/lib/markdown";
import type { messages } from "./ReleaseNoteDialog.messages";

/** This file's own keys, the shared default action label, and the entry prose. */
type DialogT = TypedT<typeof messages & typeof sharedMessages & typeof entryMessages>;

export function ReleaseNoteDialog(): React.ReactElement | null {
    // The store must be initialised here and not only in the bell: SiteChrome
    // drops the Header on /admin, so the button is not always mounted.
    useEffect(() => {
        initReleaseNotes();
    }, []);

    const t: DialogT = useT("changelog");
    const locale = useLocale();

    const open = useSelector(releaseNoteStore, (s) => s.dialogOpen);
    const missed = useSelector(releaseNoteStore, (s) => s.missed);
    const note = LATEST_ANNOUNCED_NOTE;

    if (note === null) return null;

    return (
        <Dialog open={open} onOpenChange={setReleaseNoteDialogOpen}>
            <DialogPopup className="sm:max-w-lg" closeProps={note.media ? { className: "absolute end-2 top-2 bg-background/70 backdrop-blur-sm" } : undefined}>
                {/* 16:9 costs 211px of a 375px phone before a word of the note is read. */}
                {note.media && <img src={note.media.src} alt={t(note.media.altKey)} className="aspect-[2/1] w-full shrink-0 rounded-t-2xl border-b object-cover max-sm:aspect-[5/2] max-sm:rounded-none" />}

                <DialogHeader>
                    <span className="font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">{formatNoteDate(note.date, locale)}</span>
                    <DialogTitle>{t(note.titleKey)}</DialogTitle>
                    <DialogDescription render={<div />}>
                        <Markdown text={t(note.leadKey)} className="text-sm leading-relaxed" flush />
                    </DialogDescription>
                </DialogHeader>

                {/* The sheet is capped at the viewport, so on a phone the item list
                    was whatever was left after the media, the lead and the footer,
                    which was a few rows. It gets a real share of the screen and its
                    own scroll, rather than being the thing that absorbs every other
                    element's height. */}
                {note.items && note.items.length > 0 && (
                    <DialogPanel className="min-h-0 max-sm:max-h-[48vh] max-sm:overflow-y-auto max-sm:overscroll-contain">
                        <p className="mb-3 font-medium text-[0.69rem] text-muted-foreground uppercase tracking-[0.18em]">{t("dialog.alsoInUpdate")}</p>
                        <ReleaseNoteItems items={note.items} />
                    </DialogPanel>
                )}

                <DialogFooter className="sm:justify-between">
                    <Link to="/changelog" onClick={() => setReleaseNoteDialogOpen(false)} className="self-center text-muted-foreground text-sm no-underline transition-colors hover:text-foreground max-sm:text-center">
                        {missed > 0 ? t("dialog.allUpdates.missed", { count: missed }) : t("dialog.allUpdates")}
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
    const t: DialogT = useT("changelog");
    if (note.href) {
        return (
            <div className="flex flex-col-reverse gap-2 sm:flex-row">
                <Button variant="ghost" onClick={() => setReleaseNoteDialogOpen(false)}>
                    {t("dialog.gotIt")}
                </Button>
                <Button render={<Link to={note.href} />} onClick={() => setReleaseNoteDialogOpen(false)}>
                    {note.hrefLabelKey ? t(note.hrefLabelKey) : t("note.takeALook")}
                </Button>
            </div>
        );
    }
    return <Button onClick={() => setReleaseNoteDialogOpen(false)}>{t("dialog.gotIt")}</Button>;
}
