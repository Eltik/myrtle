import { CheckIcon, CopyIcon } from "lucide-react";
import type React from "react";
import { Fragment, useEffect, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { parseStoryText, plainStoryText } from "#/lib/story/text";
import { cn } from "#/lib/utils";
import { anchorRow } from "./backlog";
import { copyPlainText } from "./clipboard";
import type { messages } from "./reader.messages";
import { backlogJumpTarget, backlogPlainText } from "./scrub";
import { speakerColor } from "./TextBox";
import type { BacklogEntry } from "./useStoryPlayer";

export interface IBacklogDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    entries: BacklogEntry[];
    title: string;
    /** The halt the reader is on: its row is marked current and does not jump; rows past it are AHEAD. */
    currentHaltIndex: number;
    /** Replay to that halt and close. */
    onJump: (haltIndex: number) => void;
}

export function BacklogDialog({ open, onOpenChange, entries, title, currentHaltIndex, onJump }: IBacklogDialogProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const [copied, setCopied] = useState(false);
    const listRef = useRef<HTMLOListElement>(null);
    // Opening lands on the CURRENT row, centred, not on the first line of the
    // story: with the log reaching past the reader, the top is rarely where
    // they are. One frame late, because the popup's content mounts with it.
    const anchor = anchorRow(entries, currentHaltIndex);
    useEffect(() => {
        if (!open || anchor < 0) return;
        const id = window.requestAnimationFrame(() => {
            listRef.current?.querySelector<HTMLElement>(`[data-story-backlog-at="${anchor}"]`)?.scrollIntoView({ block: "center" });
        });
        return () => window.cancelAnimationFrame(id);
    }, [open, anchor]);
    /** One markup-to-words pass, used by the rows AND the copy, so what is read and what is pasted cannot drift. */
    const plain = (text: string): string => plainStoryText(parseStoryText(text));
    useEffect(() => {
        if (!copied) return;
        const id = window.setTimeout(() => setCopied(false), 1500);
        return () => window.clearTimeout(id);
    }, [copied]);

    // The button flips only on a copy that HAPPENED. A blocked clipboard and a
    // failed `execCommand` both come back false, and a "Copied" over neither is
    // the one thing a copy button must not do.
    const copy = async () => {
        if (await copyPlainText(backlogPlainText(entries, plain, title))) setCopied(true);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-2xl" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                <DialogHeader className="flex-row items-center justify-between pe-12">
                    <DialogTitle>{t("backlog.title")}</DialogTitle>
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={copy} disabled={entries.length === 0}>
                        {copied ? <CheckIcon /> : <CopyIcon />}
                        {copied ? t("backlog.copied") : t("backlog.copy")}
                    </Button>
                </DialogHeader>
                <DialogPanel className="max-h-[70dvh] overflow-y-auto">
                    {/* Chapter-title divider row. The backlog covers ONE story, so there is
                        exactly one of these; a per-`[header]` divider needs the engine to
                        surface the header text, which it currently drops. */}
                    <div className="mb-3 border-b pb-2 font-heading font-semibold text-muted-foreground text-xs uppercase tracking-wide" data-story-backlog-divider>
                        {title}
                    </div>
                    {entries.length === 0 ? <p className="text-muted-foreground text-sm">{t("backlog.empty")}</p> : null}
                    <ol ref={listRef} className="flex flex-col gap-2.5">
                        {entries.map((e, i) => {
                            const target = backlogJumpTarget(e, currentHaltIndex);
                            const current = target === null;
                            // Read before, then jumped back past: still a jump, set
                            // back so the current row stays the anchor of the list.
                            const ahead = e.haltIndex > currentHaltIndex;
                            const firstAhead = ahead && (i === 0 || entries[i - 1].haltIndex <= currentHaltIndex);
                            return (
                                <Fragment key={`${e.haltIndex}-${e.kind}`}>
                                    {firstAhead ? (
                                        <li aria-hidden className="flex items-center gap-2 pt-1 text-muted-foreground text-xs uppercase tracking-wide" data-story-backlog-ahead-divider>
                                            <span className="h-px flex-1 bg-border" />
                                            {t("backlog.ahead")}
                                            <span className="h-px flex-1 bg-border" />
                                        </li>
                                    ) : null}
                                    <li data-story-backlog-at={i}>
                                        {/* Every row is a jump. The CURRENT row is not: its
                                        target is null, so it renders disabled rather than
                                        replaying the whole story to the line already shown. */}
                                        <button
                                            type="button"
                                            data-story-backlog-row
                                            data-story-halt-index={e.haltIndex}
                                            data-story-current={current ? "true" : "false"}
                                            data-story-ahead={ahead ? "true" : undefined}
                                            disabled={current}
                                            aria-current={current ? "true" : undefined}
                                            title={current ? undefined : t("backlog.jump", { line: e.haltIndex + 1 })}
                                            onClick={() => target !== null && onJump(target)}
                                            className={cn(
                                                // The backlog is a READING surface, not a list of
                                                // labels: its rows are the lines of the story
                                                // read back. They set 16 px on a phone, which is
                                                // what the text box itself sets, and keep the
                                                // list's own 14 px from 640 up.
                                                "w-full rounded-md px-2 py-1 text-start text-base leading-relaxed transition-colors sm:text-sm",
                                                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring max-sm:min-h-11",
                                                e.kind === "choice" && "bg-muted/60 font-medium",
                                                e.kind === "cutscene" && "bg-muted/40 text-muted-foreground italic",
                                                current ? "bg-primary/10 ring-1 ring-primary/40" : "hover:bg-muted",
                                                ahead && "text-muted-foreground opacity-75 hover:text-foreground hover:opacity-100",
                                            )}
                                        >
                                            {e.kind === "choice" ? (
                                                t("backlog.choice", { text: plain(e.text) })
                                            ) : e.kind === "cutscene" ? (
                                                e.text
                                            ) : (
                                                <>
                                                    <span className="me-2 font-semibold" style={e.speaker ? { color: speakerColor(e.speaker) } : undefined}>
                                                        {e.speaker ?? (e.isNarration ? "" : t("reader.narrator"))}
                                                    </span>
                                                    <span className={cn(e.isNarration && "text-muted-foreground italic")}>{plain(e.text)}</span>
                                                </>
                                            )}
                                        </button>
                                    </li>
                                </Fragment>
                            );
                        })}
                    </ol>
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}
