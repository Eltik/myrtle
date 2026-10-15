import type React from "react";
import { useEffect, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

/** The longest playlist name the dialog accepts. */
export const PLAYLIST_NAME_MAX = 60;

/**
 * The small name dialog for a new playlist or a rename. Enter submits, Escape
 * and Cancel close, an empty name cannot be submitted.
 */
export function PlaylistNameDialog({ open, onOpenChange, mode, initial = "", onSubmit }: { open: boolean; onOpenChange: (open: boolean) => void; mode: "create" | "rename"; initial?: string; onSubmit: (name: string) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [name, setName] = useState(initial);
    useEffect(() => {
        if (open) setName(initial);
    }, [open, initial]);
    const clean = name.trim();
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="sm:max-w-sm">
                {/* The form is the popup's flex column: as a plain block, the panel's full-height scroll area pushed the footer out of the card. */}
                <form
                    className="flex min-h-0 flex-col"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (!clean) return;
                        onSubmit(clean);
                        onOpenChange(false);
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>{mode === "create" ? t("music.playlist.newTitle") : t("music.playlist.renameTitle")}</DialogTitle>
                    </DialogHeader>
                    <DialogPanel>
                        <Input autoFocus value={name} maxLength={PLAYLIST_NAME_MAX} onChange={(e) => setName(e.target.value)} placeholder={t("music.playlist.namePlaceholder")} aria-label={t("music.playlist.nameLabel")} />
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>{t("music.playlist.cancel")}</DialogClose>
                        <Button type="submit" disabled={!clean}>
                            {mode === "create" ? t("music.playlist.create") : t("music.playlist.save")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
