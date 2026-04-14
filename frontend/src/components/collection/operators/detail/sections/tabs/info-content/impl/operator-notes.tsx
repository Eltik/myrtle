"use client";

import { ChevronDown, Lightbulb, Loader2, MessageSquareText, StickyNote, ThumbsDown, ThumbsUp } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Badge } from "~/components/ui/shadcn/badge";
import { Separator } from "~/components/ui/shadcn/separator";
import { useOperatorNotes } from "~/hooks/use-operator-notes";

interface OperatorNotesProps {
    operatorId: string | null;
}

export function OperatorNotes({ operatorId }: OperatorNotesProps) {
    const { notes, isLoading } = useOperatorNotes(operatorId);
    const [showNotes, setShowNotes] = useState(true);

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-4">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (!notes) {
        return null;
    }

    const hasPros = notes.pros.trim().length > 0;
    const hasCons = notes.cons.trim().length > 0;
    const hasNotes = notes.notes.trim().length > 0;
    const hasTrivia = notes.trivia.trim().length > 0;
    const hasSummary = notes.summary.trim().length > 0;
    const hasTags = notes.tags.length > 0;

    if (!hasPros && !hasCons && !hasNotes && !hasTrivia && !hasSummary && !hasTags) {
        return null;
    }

    return (
        <>
            <Separator className="my-6" />

            <Disclosure onOpenChange={setShowNotes} open={showNotes} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                <DisclosureTrigger>
                    <div className="flex w-full cursor-pointer items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                        <div className="flex items-center gap-2">
                            <MessageSquareText className="h-4 w-4 text-primary" />
                            <span className="font-medium text-sm">Operator Notes</span>
                        </div>
                        <motion.div animate={{ rotate: showNotes ? 180 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </div>
                </DisclosureTrigger>
                <DisclosureContent>
                    <div className="mt-3 space-y-4">
                        {/* Summary */}
                        {hasSummary && (
                            <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                <p className="text-foreground text-sm">{notes.summary}</p>
                            </div>
                        )}

                        {/* Pros / Cons */}
                        {(hasPros || hasCons) && (
                            <div className="grid gap-3 sm:grid-cols-2">
                                {hasPros && (
                                    <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                        <div className="mb-2 flex items-center gap-2">
                                            <ThumbsUp className="h-4 w-4 text-green-500" />
                                            <span className="font-medium text-sm">Pros</span>
                                        </div>
                                        <p className="whitespace-pre-line text-muted-foreground text-sm">{notes.pros}</p>
                                    </div>
                                )}
                                {hasCons && (
                                    <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                        <div className="mb-2 flex items-center gap-2">
                                            <ThumbsDown className="h-4 w-4 text-red-500" />
                                            <span className="font-medium text-sm">Cons</span>
                                        </div>
                                        <p className="whitespace-pre-line text-muted-foreground text-sm">{notes.cons}</p>
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Notes */}
                        {hasNotes && (
                            <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                <div className="mb-2 flex items-center gap-2">
                                    <StickyNote className="h-4 w-4 text-yellow-500" />
                                    <span className="font-medium text-sm">Notes</span>
                                </div>
                                <p className="whitespace-pre-line text-muted-foreground text-sm">{notes.notes}</p>
                            </div>
                        )}

                        {/* Trivia */}
                        {hasTrivia && (
                            <div className="rounded-lg border border-border bg-secondary/20 p-4">
                                <div className="mb-2 flex items-center gap-2">
                                    <Lightbulb className="h-4 w-4 text-amber-500" />
                                    <span className="font-medium text-sm">Trivia</span>
                                </div>
                                <p className="whitespace-pre-line text-muted-foreground text-sm">{notes.trivia}</p>
                            </div>
                        )}

                        {/* Tags */}
                        {hasTags && (
                            <div className="flex flex-wrap gap-2">
                                {notes.tags.map((tag) => (
                                    <Badge key={tag} variant="secondary">
                                        {tag}
                                    </Badge>
                                ))}
                            </div>
                        )}
                    </div>
                </DisclosureContent>
            </Disclosure>
        </>
    );
}
