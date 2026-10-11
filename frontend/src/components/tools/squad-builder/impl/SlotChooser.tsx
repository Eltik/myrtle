import type * as React from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { SavedSquad } from "#/types/generated/SavedSquad";
import { SLOT_LIMIT } from "../model";
import type { messages } from "./SlotChooser.messages";

export interface ISlotChooserProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    saved: ReadonlyArray<SavedSquad | null>;
    busy: boolean;
    onPick: (slot: number) => void;
    /** Shown when copying a shared squad: keep it unsaved. */
    onEditCopy?: () => void;
}

export function SlotChooser({ open, onOpenChange, saved, busy, onPick, onEditCopy }: ISlotChooserProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{t("squad.chooser.title")}</DialogTitle>
                    <DialogDescription>{t("squad.chooser.desc", { count: SLOT_LIMIT })}</DialogDescription>
                </DialogHeader>
                <DialogPanel>
                    <ul className="grid grid-cols-2 gap-2">
                        {Array.from({ length: SLOT_LIMIT }, (_, i) => {
                            const slot = i + 1;
                            const squad = saved[i] ?? null;
                            return (
                                <li key={`choose-${slot}`}>
                                    <button
                                        type="button"
                                        disabled={busy}
                                        onClick={() => onPick(slot)}
                                        className="flex w-full cursor-pointer flex-col gap-0.5 rounded-lg border border-border bg-background/40 p-2.5 text-left outline-none transition-colors hover:border-primary/60 focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-wait disabled:opacity-60"
                                    >
                                        <span className="font-semibold text-[13px]">{t("squad.chooser.slot", { slot })}</span>
                                        <span className="truncate text-[12px] text-muted-foreground">{squad ? t("squad.chooser.replaces", { name: squad.name.trim() || t("squad.chooser.untitled", { slot }) }) : t("squad.chooser.empty")}</span>
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                </DialogPanel>
                {onEditCopy && (
                    <DialogFooter>
                        <Button variant="outline" onClick={onEditCopy}>
                            {t("squad.chooser.editCopy")}
                        </Button>
                    </DialogFooter>
                )}
            </DialogPopup>
        </Dialog>
    );
}
