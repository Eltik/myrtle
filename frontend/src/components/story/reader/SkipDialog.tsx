/**
 * The Skip sheet, in the client's manner: the story's name, the "Story
 * summary" label, the game's own summary, and two answers. Confirming goes
 * straight to the end card (`useStoryPlayer.skipToEnd`); nothing steps.
 *
 * `useT` is bound here rather than taken as a prop, for the reason
 * `ReaderToolbar` gives: the i18n extractor binds a `t` by the `useT` call in
 * the same file.
 */
import { FastForwardIcon } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { parseStoryText } from "#/lib/story/text";
import type { messages } from "./reader.messages";
import { synopsisParagraphs } from "./skip";
import { renderStoryNodes } from "./TextBox";

export interface ISkipDialogProps {
    open: boolean;
    /** Closing by Escape, the backdrop or the close button is "Keep reading". */
    onCancel: () => void;
    onConfirm: () => void;
    title: string;
    /** The script's `synopsis`; absent when the game ships none for this story. */
    synopsis?: string;
}

export function SkipDialog({ open, onCancel, onConfirm, title, synopsis }: ISkipDialogProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const paragraphs = synopsisParagraphs(synopsis);
    return (
        <Dialog open={open} onOpenChange={(next: boolean) => !next && onCancel()}>
            <DialogPopup className="max-w-xl" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }} data-story-skip-sheet>
                <DialogHeader className="pe-12">
                    {/* The label sits ABOVE the name, the way the client prints
                        its summary card: a small caps kicker, then the title. */}
                    <span className="font-heading font-semibold text-muted-foreground text-xs uppercase tracking-wide">{t("reader.skip.summaryLabel")}</span>
                    <DialogTitle className="text-balance">{title}</DialogTitle>
                </DialogHeader>
                <DialogPanel className="max-h-[50dvh] overflow-y-auto">
                    {paragraphs.length > 0 ? (
                        <div className="flex flex-col gap-3 text-base leading-relaxed sm:text-sm" data-story-synopsis>
                            {paragraphs.map((p, i) => (
                                // A paragraph's index IS its identity: the list never reorders.
                                // biome-ignore lint/suspicious/noArrayIndexKey: static paragraphs of one synopsis
                                <p key={i}>{renderStoryNodes(parseStoryText(p))}</p>
                            ))}
                        </div>
                    ) : (
                        <DialogDescription data-story-synopsis-none>{t("reader.skip.none")}</DialogDescription>
                    )}
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="outline" className="max-sm:h-11" />}>{t("reader.skip.cancel")}</DialogClose>
                    <Button className="max-sm:h-11" onClick={onConfirm} data-story-skip-confirm>
                        <FastForwardIcon />
                        {t("reader.skip.confirm")}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
