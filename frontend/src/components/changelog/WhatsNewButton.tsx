"use client";

import { useSelector } from "@tanstack/react-store";
import { BellIcon } from "lucide-react";
import type React from "react";
import { useEffect } from "react";
import { Button } from "#/components/ui/button";
import { LATEST_ANNOUNCED_NOTE } from "#/content/changelog/entries";
import { hasUnseenReleaseNote, initReleaseNotes, openReleaseNoteDialog, releaseNoteStore } from "#/lib/changelog/store";

/**
 * The ONLY entry point to the announcement. Nothing opens on its own: the dot is
 * the whole notification, and the click that opens the dialog is what clears it.
 */
export function WhatsNewButton(): React.ReactElement | null {
    const unseen = useSelector(releaseNoteStore, hasUnseenReleaseNote);

    useEffect(() => {
        initReleaseNotes();
    }, []);

    if (LATEST_ANNOUNCED_NOTE === null) return null;

    return (
        <Button aria-label={unseen ? "What's new (unread)" : "What's new"} className="relative" onClick={openReleaseNoteDialog} size="icon" variant="ghost">
            <BellIcon className="h-4 w-4" aria-hidden="true" />
            {unseen && <span aria-hidden="true" className="absolute end-1.5 top-1.5 size-2 rounded-full bg-primary ring-2 ring-background" />}
        </Button>
    );
}
