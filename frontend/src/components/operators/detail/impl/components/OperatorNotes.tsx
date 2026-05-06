import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Lightbulb, Loader2, MessageSquareText, StickyNote, ThumbsDown, ThumbsUp } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { operatorNoteQueryOptions } from "#/lib/api/operator-notes";
import { cn } from "#/lib/utils";

interface IOperatorNotesProps {
    operatorId: string | null;
}

const COLLAPSED_MAX_HEIGHT = 120;

function ExpandableText({ text }: { text: string }) {
    const ref = useRef<HTMLParagraphElement>(null);
    const [overflows, setOverflows] = useState(false);
    const [expanded, setExpanded] = useState(false);

    useLayoutEffect(() => {
        const el = ref.current;
        if (!el) return;
        const measure = () => setOverflows(el.scrollHeight > COLLAPSED_MAX_HEIGHT + 1);
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    return (
        <div>
            <div className="relative">
                <p ref={ref} className={cn("whitespace-pre-line text-muted-foreground text-sm transition-[max-height] duration-200", !expanded && overflows && "overflow-hidden")} style={!expanded && overflows ? { maxHeight: COLLAPSED_MAX_HEIGHT } : undefined}>
                    {text}
                </p>
                {!expanded && overflows && <div aria-hidden className="pointer-events-none absolute inset-x-0 bottom-0 h-10 bg-gradient-to-t from-secondary/40 to-transparent" />}
            </div>
            {overflows && (
                <button className="mt-2 inline-flex items-center gap-1 font-medium text-primary text-xs hover:underline" onClick={() => setExpanded((v) => !v)} type="button">
                    {expanded ? "Show less" : "Show more"}
                    <ChevronDown className={cn("h-3 w-3 transition-transform", expanded && "rotate-180")} />
                </button>
            )}
        </div>
    );
}

export function OperatorNotes({ operatorId }: IOperatorNotesProps) {
    const { data: note, isLoading } = useQuery({
        ...operatorNoteQueryOptions(operatorId ?? ""),
        enabled: !!operatorId,
    });
    const [showNotes, setShowNotes] = useState(true);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-4">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (!note) return null;

    const summary = note.summary?.trim() ?? "";
    const pros = note.pros?.trim() ?? "";
    const cons = note.cons?.trim() ?? "";
    const notesText = note.notes?.trim() ?? "";
    const trivia = note.trivia?.trim() ?? "";
    const tags = note.tags ?? [];

    const hasSummary = summary.length > 0;
    const hasPros = pros.length > 0;
    const hasCons = cons.length > 0;
    const hasNotes = notesText.length > 0;
    const hasTrivia = trivia.length > 0;
    const hasTags = tags.length > 0;

    if (!hasSummary && !hasPros && !hasCons && !hasNotes && !hasTrivia && !hasTags) return null;

    return (
        <Collapsible open={showNotes} onOpenChange={setShowNotes}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <MessageSquareText className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">Operator Notes</span>
                </span>
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="mt-3 space-y-4">
                    {hasSummary && (
                        <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                            <ExpandableText text={summary} />
                        </div>
                    )}

                    {(hasPros || hasCons) && (
                        <div className="grid gap-3 sm:grid-cols-2">
                            {hasPros && (
                                <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                    <div className="mb-2 flex items-center gap-2">
                                        <ThumbsUp className="h-4 w-4 text-green-500" />
                                        <span className="font-medium text-sm">Pros</span>
                                    </div>
                                    <ExpandableText text={pros} />
                                </div>
                            )}
                            {hasCons && (
                                <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                    <div className="mb-2 flex items-center gap-2">
                                        <ThumbsDown className="h-4 w-4 text-red-500" />
                                        <span className="font-medium text-sm">Cons</span>
                                    </div>
                                    <ExpandableText text={cons} />
                                </div>
                            )}
                        </div>
                    )}

                    {hasNotes && (
                        <div className="rounded-lg border border-border bg-secondary/20 p-4">
                            <div className="mb-2 flex items-center gap-2">
                                <StickyNote className="h-4 w-4 text-yellow-500" />
                                <span className="font-medium text-sm">Notes</span>
                            </div>
                            <ExpandableText text={notesText} />
                        </div>
                    )}

                    {hasTrivia && (
                        <div className="rounded-lg border border-border bg-secondary/20 p-4">
                            <div className="mb-2 flex items-center gap-2">
                                <Lightbulb className="h-4 w-4 text-amber-500" />
                                <span className="font-medium text-sm">Trivia</span>
                            </div>
                            <ExpandableText text={trivia} />
                        </div>
                    )}

                    {hasTags && (
                        <div className="flex flex-wrap gap-2">
                            {tags.map((tag) => (
                                <Badge key={tag} variant="secondary">
                                    {tag}
                                </Badge>
                            ))}
                        </div>
                    )}
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}
