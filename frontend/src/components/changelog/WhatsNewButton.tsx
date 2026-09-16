"use client";

import { useSelector } from "@tanstack/react-store";
import { BellIcon } from "lucide-react";
import type React from "react";
import { useEffect } from "react";
import { Button } from "#/components/ui/button";
import { LATEST_ANNOUNCED_NOTE } from "#/content/changelog/entries";
import { hasUnseenReleaseNote, initReleaseNotes, openReleaseNoteDialog, releaseNoteStore } from "#/lib/changelog/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./WhatsNewButton.messages";

/**
 * The ONLY entry point to the announcement. Nothing opens on its own: the dot is
 * the whole notification, and the click that opens the dialog is what clears it.
 */
export function WhatsNewButton(): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("changelog");
    const unseen = useSelector(releaseNoteStore, hasUnseenReleaseNote);

    useEffect(() => {
        initReleaseNotes();
    }, []);

    if (LATEST_ANNOUNCED_NOTE === null) return null;

    return (
        // `min-[300px]`: the header's action cluster is five tap targets and
        // fits from 300px up once the wordmark yields (measured: 300px is the
        // exact break-even, signed-in and signed-out). Below that - the Fold's
        // 280px cover screen, essentially nothing else - this is the control
        // whose loss costs least, and the changelog is still one tap away in
        // the drawer.
        <Button aria-label={unseen ? t("whatsNew.aria.unread") : t("whatsNew.aria")} className="relative hidden min-[300px]:inline-flex" onClick={openReleaseNoteDialog} size="icon" variant="ghost">
            <BellIcon className="h-4 w-4" aria-hidden="true" />
            {unseen && <span aria-hidden="true" className="absolute end-1.5 top-1.5 size-2 rounded-full bg-primary ring-2 ring-background" />}
        </Button>
    );
}
