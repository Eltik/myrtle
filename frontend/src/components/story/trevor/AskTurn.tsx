/** One turn of the Ask chat: the reader's question and Trevor's reply in whatever state the question is in. */
import type React from "react";
import { useId } from "react";
import { ChatMessage } from "#/components/chat/chat-message";
import { ChatTypingIndicator } from "#/components/chat/chat-typing-indicator";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IParsedAnswer } from "#/lib/trevor/answer";
import { askPending, hasJob } from "#/lib/trevor/askState";
import type { TrevorFailureKind } from "#/lib/trevor/errors";
import { MAX_OPEN, type ThreadTurn, type TurnState } from "#/lib/trevor/thread";
import { AnswerText } from "./AnswerText";
import { AnswerFooter } from "./AnswerView";
import { TrevorAvatar } from "./TrevorAvatar";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

/** One turn of the thread: an asked question and its reply, or a prepared answer opened from a chip. */
export function ChatTurn({ turn, latest, storyId, signedIn, onJumpLine, onResume }: { turn: ThreadTurn; latest: boolean; storyId: string | null; signedIn: boolean; onJumpLine?: (line: number) => void; onResume: (turnId: number) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    if (turn.kind === "ask") return <AskTurn state={turn.state} latest={latest} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} onResume={() => onResume(turn.id)} />;
    return (
        <div className="flex flex-col gap-4" data-trevor-turn="bank">
            <ChatMessage side="outgoing">{turn.question}</ChatMessage>
            <ChatMessage side="incoming" author="Trevor" avatar={<TrevorAvatar />} footer={<span className="text-muted-foreground text-xs">{t("trevor.suggest.prepared")}</span>} data-trevor-bank-answer>
                <AnswerText text={turn.answer} sourceId={null} cited={new Set()} />
            </ChatMessage>
        </div>
    );
}

/** One asked question and Trevor's reply; `latest` marks the newest asked turn. */
function AskTurn({ state, latest, storyId, signedIn, onJumpLine, onResume }: { state: TurnState; latest: boolean; storyId: string | null; signedIn: boolean; onJumpLine?: (line: number) => void; onResume: () => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const describe = useErrorMessage();
    const sourceId = `trevor-src-${useId().replace(/:/g, "")}`;
    const waiting = state.kind === "sending" || state.kind === "queued";
    const pending = askPending(state);
    const author = "Trevor";
    const avatar = <TrevorAvatar />;

    const quick = hasJob(state) ? state.quick : undefined;
    let reply: React.ReactNode;
    if (state.kind === "held") {
        reply = (
            <ChatMessage side="incoming" author={author} avatar={avatar} bubble={false}>
                <p className="text-muted-foreground text-sm" data-trevor-held>
                    {t("trevor.chat.held", { max: MAX_OPEN })}
                </p>
            </ChatMessage>
        );
    } else if (waiting) {
        const label = state.kind === "sending" ? t("trevor.ask.sending") : [quick ? t("trevor.quick.writing") : t("trevor.ask.queued"), eta(state.etaSecs, t), state.position ? t("trevor.ask.position", { position: state.position }) : null].filter(Boolean).join(" · ");
        reply = (
            <ChatMessage side="incoming" author={author} avatar={avatar} bubble={false} footer={state.kind === "queued" && !state.workerOnline ? <span className="text-muted-foreground text-xs">{t("trevor.ask.noWorker")}</span> : undefined}>
                <ChatTypingIndicator label={label} />
            </ChatMessage>
        );
    } else if (state.kind === "streaming") {
        reply = (
            <ChatMessage side="incoming" author={author} avatar={avatar}>
                <AnswerText text={state.text} sourceId={null} cited={new Set()} />
            </ChatMessage>
        );
    } else if (state.kind === "done" && state.quick) {
        reply = <AnswerReply answer={state.answer} quick note={state.fullFailed ? t("trevor.quick.fullFailed") : null} sourceId={sourceId} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} />;
    } else if (state.kind === "done") {
        const passages = state.answer.source === "passages";
        reply = <AnswerReply answer={state.answer} passagesLabel={passages} note={passages ? t("trevor.ask.passagesNote") : null} sourceId={sourceId} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} />;
    } else {
        // failed, refused, offline: Trevor says so in words, as a quieter reply.
        const words = state.kind === "failed" ? t("trevor.ask.failed") : state.kind === "offline" ? t("trevor.ask.offline") : (refusal(state.failure, t) ?? describe(state.error));
        const action =
            state.kind === "offline" ? (
                <Button variant="outline" size="sm" className="max-sm:h-11" onClick={onResume}>
                    {t("trevor.ask.checkAgain")}
                </Button>
            ) : state.kind === "refused" && state.failure === "signedOut" ? (
                <Button size="sm" className="max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                    {t("trevor.ask.signIn")}
                </Button>
            ) : undefined;
        reply = (
            <ChatMessage side="incoming" author={author} avatar={avatar} footer={action} bubbleProps={{ className: "border border-warning/32 bg-warning/8" }}>
                <p className="text-sm" data-trevor-refused={state.kind === "refused" ? state.failure : undefined}>
                    {words}
                </p>
            </ChatMessage>
        );
    }

    return (
        <div className="flex flex-col gap-4" data-trevor-turn="ask">
            <ChatMessage side="outgoing" animateEntrance={state.kind === "sending" || state.kind === "held"} data-trevor-asked>
                {state.question}
            </ChatMessage>
            {/* The quick look is an answer already: its own region, read at once, while the full answer is still busy below it. */}
            {quick ? (
                <div aria-live="polite">
                    <AnswerReply answer={quick} quick note={t("trevor.quick.deferred")} sourceId={`${sourceId}-q`} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} />
                </div>
            ) : null}
            <div aria-live="polite" aria-busy={pending} data-trevor-state={state.kind} data-trevor-live={latest ? "" : undefined}>
                {reply}
            </div>
        </div>
    );
}

/**
 * A finished answer as Trevor's reply. A quick look (a passages answer given at
 * once) is always labelled "passages only"; a done answer is when its source is
 * passages. `note` sits under the label.
 */
function AnswerReply({
    answer,
    quick = false,
    passagesLabel = quick,
    note,
    sourceId,
    storyId,
    signedIn,
    onJumpLine,
}: {
    answer: IParsedAnswer;
    quick?: boolean;
    passagesLabel?: boolean;
    note: string | null;
    sourceId: string;
    storyId: string | null;
    signedIn: boolean;
    onJumpLine?: (line: number) => void;
}): React.ReactElement {
    const t: TrevorT = useT("story");
    return (
        <ChatMessage side="incoming" author="Trevor" avatar={<TrevorAvatar />} footer={<AnswerFooter answer={answer} sourceId={sourceId} storyId={storyId} onJumpLine={onJumpLine} signedIn={signedIn} />} aria-label={t("trevor.answer.label")} data-trevor-quick={quick || undefined}>
            <div className="flex flex-col gap-2">
                {passagesLabel ? (
                    <div className="flex flex-col gap-1">
                        <span className="self-start rounded-full bg-background/70 px-2 py-0.5 font-medium text-xs" data-trevor-passages>
                            {t("trevor.ask.passagesOnly")}
                        </span>
                        {note ? <p className="text-muted-foreground text-xs">{note}</p> : null}
                    </div>
                ) : null}
                {answer.text ? <AnswerText text={answer.text} sourceId={sourceId} cited={new Set(answer.citations.map((c) => c.n))} /> : null}
            </div>
        </ChatMessage>
    );
}

function eta(secs: number | null, t: TrevorT): string | null {
    if (secs === null || !Number.isFinite(secs) || secs <= 0) return null;
    return secs < 90 ? t("trevor.ask.etaSeconds", { count: Math.round(secs) }) : t("trevor.ask.etaMinutes", { count: Math.round(secs / 60) });
}

function refusal(kind: TrevorFailureKind, t: TrevorT): string | null {
    switch (kind) {
        case "signedOut":
            return t("trevor.ask.sessionEnded");
        case "verify":
            return t("trevor.ask.verify");
        case "tooMany":
            return t("trevor.ask.tooMany");
        case "tooLong":
            return t("trevor.ask.tooLong");
        case "rateLimited":
            return t("trevor.ask.rateLimited");
        case "busy":
            return t("trevor.ask.busy");
        case "other":
            return null;
    }
}
