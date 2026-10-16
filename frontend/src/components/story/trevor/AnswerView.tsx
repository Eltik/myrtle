/**
 * A finished answer: its spoiler label, the passages-only label when no model
 * wrote it, the text, the cited passages as links into the reader, and the
 * thumbs. A citation in the story being read jumps the reader in place (the
 * route keeps one reader per story, so a `?line=` link to the same story
 * would not move it); any other story opens at the cited line.
 */
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ThumbsDownIcon, ThumbsUpIcon } from "lucide-react";
import type React from "react";
import { useId, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { sendFeedback } from "#/lib/api/trevor";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IParsedAnswer, ITrevorCitation } from "#/lib/trevor/answer";
import { storyLabel } from "#/lib/trevor/stories";
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import { AnswerText } from "./AnswerText";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

export function SpoilerLabel({ spoiler }: { spoiler: TrevorSpoilerSafe | null }): React.ReactElement {
    const t: TrevorT = useT("story");
    return (
        <Badge variant="outline" className="h-auto max-w-full whitespace-normal py-0.5 text-start" data-trevor-spoiler>
            {spoiler ? t("trevor.ask.spoiler", { era: spoiler.name }) : t("trevor.ask.spoilerNone")}
        </Badge>
    );
}

export interface IAnswerViewProps {
    answer: IParsedAnswer;
    spoiler: TrevorSpoilerSafe | null;
    /** The story the reader is on. */
    storyId: string | null;
    /** Jump the open reader to a script line (citations in the current story). */
    onJumpLine?: (line: number) => void;
    signedIn: boolean;
}

export function AnswerView({ answer, spoiler, storyId, onJumpLine, signedIn }: IAnswerViewProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const sourceId = useId().replace(/:/g, "");
    const cited = new Set(answer.citations.map((c) => c.n));
    const passages = answer.source === "passages";
    return (
        <section aria-label={t("trevor.answer.label")} className="flex flex-col gap-3" data-trevor-answer data-trevor-source={answer.source}>
            <div className="flex flex-wrap items-center gap-1.5">
                <SpoilerLabel spoiler={spoiler} />
                {passages ? (
                    <Badge variant="secondary" data-trevor-passages>
                        {t("trevor.ask.passagesOnly")}
                    </Badge>
                ) : null}
            </div>
            {passages ? <p className="text-muted-foreground text-xs">{t("trevor.ask.passagesNote")}</p> : null}
            {answer.text ? <AnswerText text={answer.text} sourceId={sourceId} cited={cited} /> : null}
            {answer.citations.length > 0 ? <Sources citations={answer.citations} sourceId={sourceId} storyId={storyId} onJumpLine={onJumpLine} /> : null}
            {signedIn ? <Feedback answerId={answer.id} /> : null}
        </section>
    );
}

function Sources({ citations, sourceId, storyId, onJumpLine }: { citations: ITrevorCitation[]; sourceId: string; storyId: string | null; onJumpLine?: (line: number) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const server = useGamedataServer();
    // The reader route loaded the index already; this is a cache read.
    const { data: index } = useQuery({ ...storyIndexQueryOptions(server), enabled: citations.length > 0 });
    return (
        <div className="flex flex-col gap-2">
            <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">{t("trevor.answer.sources")}</h4>
            <ol className="flex flex-col gap-2">
                {citations.map((c) => {
                    const label = storyLabel(index, c.storyId);
                    const where = label ? [label.groupName, label.code, label.name].filter(Boolean).join(" · ") : t("trevor.answer.otherSource");
                    const line = c.lineStart > 0 ? t("trevor.answer.line", { line: c.lineStart }) : null;
                    const linkClass = "font-medium text-foreground text-xs underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-sm";
                    const heading = (
                        <>
                            {where}
                            {line ? <span className="text-muted-foreground"> · {line}</span> : null}
                        </>
                    );
                    return (
                        <li key={`${c.n}-${c.chunkId}`} id={`${sourceId}-${c.n}`} className="flex gap-2 rounded-md border border-border bg-muted/40 p-2.5" data-trevor-citation={c.n}>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">[{c.n}]</span>
                            <div className="flex min-w-0 flex-col gap-1">
                                {label && c.storyId === storyId && onJumpLine && c.lineStart > 0 ? (
                                    <button type="button" className={`${linkClass} text-start`} title={t("trevor.answer.openHere")} onClick={() => onJumpLine(c.lineStart)}>
                                        {heading}
                                    </button>
                                ) : label ? (
                                    <Link to="/stories/$storyId" params={{ storyId: c.storyId }} search={c.lineStart > 0 ? { line: c.lineStart } : {}} className={linkClass}>
                                        {heading}
                                    </Link>
                                ) : (
                                    <span className="text-muted-foreground text-xs">{heading}</span>
                                )}
                                {c.text ? <blockquote className="line-clamp-4 whitespace-pre-line text-muted-foreground text-xs leading-relaxed">{c.text}</blockquote> : null}
                            </div>
                        </li>
                    );
                })}
            </ol>
        </div>
    );
}

function Feedback({ answerId }: { answerId: string }): React.ReactElement {
    const t: TrevorT = useT("story");
    const describe = useErrorMessage();
    const [vote, setVote] = useState<-1 | 0 | 1>(0);
    const [sent, setSent] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const cast = async (v: -1 | 1) => {
        const next = vote === v ? 0 : v;
        setError(null);
        try {
            await sendFeedback(answerId, next);
            setVote(next);
            setSent(next !== 0);
        } catch (err) {
            setError(describe(err));
        }
    };
    return (
        <div className="flex flex-wrap items-center gap-2 text-muted-foreground text-xs">
            <span id={`trevor-fb-${answerId}`}>{t("trevor.feedback.label")}</span>
            <fieldset aria-labelledby={`trevor-fb-${answerId}`} className="m-0 flex gap-1 border-0 p-0">
                <Button variant="ghost" size="icon-sm" aria-label={t("trevor.feedback.up")} aria-pressed={vote === 1} onClick={() => void cast(1)} className="max-sm:size-11">
                    <ThumbsUpIcon />
                </Button>
                <Button variant="ghost" size="icon-sm" aria-label={t("trevor.feedback.down")} aria-pressed={vote === -1} onClick={() => void cast(-1)} className="max-sm:size-11">
                    <ThumbsDownIcon />
                </Button>
            </fieldset>
            <output className="min-h-4">{error ?? (sent ? t("trevor.feedback.thanks") : "")}</output>
        </div>
    );
}
