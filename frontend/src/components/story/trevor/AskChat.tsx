/**
 * THE ASK TAB AS A CHAT (coss chat: `components/chat/*`, `ui/composer`).
 *
 * The reader's questions are outgoing messages and Trevor's replies incoming
 * ones, in the order asked, in a log that follows the newest message while
 * the reader is at the bottom and offers "Jump to latest" when they scrolled
 * up. The reader may ask again while earlier questions are still queued or
 * streaming: each turn has its own job and watcher (`useTrevorThread`,
 * `lib/trevor/thread.ts`), and answers stream into their own bubbles at once.
 * Past the backend's 3 open questions a turn waits here, said so in words,
 * and is sent when one is answered; the composer never stands down for it.
 *
 * Each reply carries its own live region (the log itself is `aria-live=off`):
 * polite, and busy while the answer streams, so a screen reader reads it once,
 * whole, instead of word by word. When an answer settles and focus was lost
 * (on the page body), it returns to the composer. The empty state offers the panel's suggested
 * questions as chips: one with a prepared answer adds it to the thread with no
 * request; one without asks Trevor.
 */
import { ArrowUpIcon } from "lucide-react";
import type React from "react";
import { useEffect, useId, useRef, useState } from "react";
import { ChatConversation } from "#/components/chat/chat-conversation";
import { ChatMessage } from "#/components/chat/chat-message";
import { ChatTypingIndicator } from "#/components/chat/chat-typing-indicator";
import { useChatScroll } from "#/components/chat/hooks/use-chat-scroll";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { Avatar, AvatarFallback } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Composer, ComposerActionGroup, ComposerActions, ComposerInput, ComposerSubmit, type ComposerValue, getComposerText } from "#/components/ui/composer";
import { useErrorMessage } from "#/components/ui/error-message";
import { useMediaQuery } from "#/hooks/use-media-query";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { askPending } from "#/lib/trevor/askState";
import type { TrevorFailureKind } from "#/lib/trevor/errors";
import { turnstileSiteKey } from "#/lib/trevor/flag";
import type { TrevorPanel, TrevorSuggestion } from "#/lib/trevor/panels";
import { type IThread, latestSpoiler, MAX_OPEN, pendingCount, type ThreadTurn, type TurnState } from "#/lib/trevor/thread";
import { cn } from "#/lib/utils";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { AnswerText } from "./AnswerText";
import { AnswerFooter, SpoilerLabel } from "./AnswerView";
import { Turnstile } from "./Turnstile";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

/** The backend's `MAX_QUESTION_CHARS`. */
export const QUESTION_MAX = 500;

export interface IAskChatProps {
    storyId: string | null;
    panel: TrevorPanel | null;
    signedIn: boolean;
    status: TrevorStatus | undefined;
    /** Every turn of the sitting, from `useTrevorThread`. */
    thread: IThread;
    onAsk: (question: string, turnstileToken: string | null) => void;
    onPrepared: (suggestion: { id: string; question: string; answer: string }) => void;
    onResume: (turnId: number) => void;
    onJumpLine?: (line: number) => void;
    /** The chat is on screen (sheet open, Ask tab selected): end-following runs only then. */
    active: boolean;
}

export function AskChat({ storyId, panel, signedIn, status, thread, onAsk, onPrepared, onResume, onJumpLine, active }: IAskChatProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const [draft, setDraft] = useState<ComposerValue>([]);
    const siteKey = turnstileSiteKey();
    const [token, setToken] = useState<string | null>(null);
    const [resetKey, setResetKey] = useState(0);
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const rootRef = useRef<HTMLDivElement>(null);
    const hintId = useId();
    const coarse = useMediaQuery("(pointer: coarse)");
    const pending = pendingCount(thread);
    const empty = thread.turns.length === 0;
    const spoiler = latestSpoiler(thread);
    const lastId = [...thread.turns].reverse().find((x) => x.kind === "ask")?.id;
    const layoutKey = thread.turns.map((x) => (x.kind === "ask" ? `${x.state.kind}${x.state.kind === "streaming" ? x.state.text.length : ""}` : "b")).join(",");
    const scroll = useChatScroll({ active, itemCount: thread.turns.length, layoutKey });

    // An answer settling hands the keyboard back to the composer when focus
    // was lost to the page body, or sat on the send button; never when the
    // reader is on something else, such as a source link in another answer.
    const wasPending = useRef(pending);
    useEffect(() => {
        if (pending < wasPending.current) {
            const focused = document.activeElement;
            const onSend = focused instanceof HTMLButtonElement && rootRef.current?.querySelector("[data-trevor-composer]")?.contains(focused);
            if (!focused || focused === document.body || onSend) inputRef.current?.focus();
        }
        wasPending.current = pending;
    }, [pending]);

    const ask = (question: string) => {
        if (!signedIn) return;
        scroll.atBottomRef.current = true;
        onAsk(question, token);
        if (siteKey) setResetKey((k) => k + 1);
    };
    const openPrepared = (s: TrevorSuggestion, answer: string) => {
        scroll.atBottomRef.current = true;
        onPrepared({ id: s.id, question: s.question, answer });
    };
    const onSuggestion = (s: TrevorSuggestion) => {
        const answer = s.answer?.trim();
        if (answer) openPrepared(s, answer);
        else if (signedIn) ask(s.question);
        else authActions.openLoginDialog();
    };

    const text = getComposerText(draft);
    const tooLong = text.length > QUESTION_MAX;
    const blockedByCheck = siteKey !== null && token === null;

    return (
        <div ref={rootRef} className="flex min-h-0 flex-1 flex-col" data-trevor-chat>
            <ChatConversation scroll={scroll} label={t("trevor.chat.label")} jumpLabel={t("trevor.chat.jump")} live="off" contentClassName="px-4 pt-4 sm:px-6">
                {empty ? (
                    <EmptyChat panel={panel} signedIn={signedIn} onSuggestion={onSuggestion} />
                ) : (
                    <div className="flex flex-col gap-4">
                        {thread.turns.map((turn) => (
                            <Turn key={turn.id} turn={turn} latest={turn.id === lastId} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} onResume={onResume} />
                        ))}
                    </div>
                )}
            </ChatConversation>
            <div className="flex shrink-0 flex-col gap-2 border-border border-t px-4 pt-3 pb-4 sm:px-6">
                {status && !status.online ? (
                    <Alert variant="warning" className="py-2">
                        <AlertDescription>{t("trevor.ask.statusOffline")}</AlertDescription>
                    </Alert>
                ) : null}
                {spoiler !== undefined ? (
                    <div className="flex">
                        <SpoilerLabel spoiler={spoiler} />
                    </div>
                ) : null}
                {signedIn ? (
                    <>
                        {siteKey ? <Turnstile siteKey={siteKey} onToken={setToken} resetKey={resetKey} /> : null}
                        <Composer
                            submitOnEnter="desktop"
                            value={draft}
                            onValueChange={setDraft}
                            submitDisabled={tooLong || blockedByCheck}
                            onSubmit={(value: ComposerValue) => {
                                const question = getComposerText(value).trim();
                                if (!question || question.length > QUESTION_MAX) return;
                                ask(question);
                                setDraft([]);
                            }}
                            data-trevor-composer
                        >
                            <ComposerInput aria-label={t("trevor.ask.label")} aria-describedby={coarse ? undefined : hintId} placeholder={t("trevor.ask.placeholder")} render={<textarea ref={inputRef} maxLength={QUESTION_MAX} data-trevor-question />} />
                            <ComposerActions>
                                <ComposerActionGroup>
                                    <span className={cn("ps-1 font-mono text-muted-foreground text-xs tabular-nums", text.length < QUESTION_MAX * 0.8 && "sr-only")} aria-hidden>
                                        {t("trevor.ask.count", { count: text.length, max: QUESTION_MAX })}
                                    </span>
                                </ComposerActionGroup>
                                <ComposerActionGroup align="end">
                                    <ComposerSubmit
                                        render={
                                            <Button aria-label={t("trevor.ask.submit")} size="icon-sm" className="rounded-full disabled:border-border disabled:bg-muted disabled:text-muted-foreground disabled:opacity-100 max-sm:size-10">
                                                <ArrowUpIcon aria-hidden />
                                            </Button>
                                        }
                                    />
                                </ComposerActionGroup>
                            </ComposerActions>
                        </Composer>
                        {coarse ? null : (
                            <p id={hintId} className="text-end text-muted-foreground text-xs">
                                {t("trevor.chat.enterHint")}
                            </p>
                        )}
                    </>
                ) : (
                    <div className="flex items-center justify-between gap-3 rounded-lg border border-border border-dashed p-3" data-trevor-signed-out>
                        <p className="text-muted-foreground text-sm">{t("trevor.ask.signedOut")}</p>
                        <Button size="sm" className="shrink-0 max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                            {t("trevor.ask.signIn")}
                        </Button>
                    </div>
                )}
            </div>
        </div>
    );
}

/** The empty thread: Trevor's one-line greeting (what it will and won't use), then the panel's suggested questions as chips. */
function EmptyChat({ panel, signedIn, onSuggestion }: { panel: TrevorPanel | null; signedIn: boolean; onSuggestion: (s: TrevorSuggestion) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const items = panel?.suggestions?.items ?? [];
    return (
        <div className="flex flex-col gap-5" data-trevor-section="suggestions">
            <ChatMessage side="incoming" author="Trevor" avatar={<TrevorAvatar />}>
                <p className="text-sm leading-relaxed">{t("trevor.ask.hint")}</p>
            </ChatMessage>
            {items.length > 0 ? (
                <section className="flex flex-col gap-2" aria-labelledby="trevor-suggested">
                    <h3 id="trevor-suggested" className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                        {t("trevor.suggest.heading")}
                    </h3>
                    <ul className="flex flex-wrap gap-2">
                        {items.map((s) => (
                            <li key={s.id} className="max-w-full" data-trevor-suggestion={s.id}>
                                <button
                                    type="button"
                                    onClick={() => onSuggestion(s)}
                                    className="max-w-full rounded-2xl border border-border bg-background px-3 py-1.5 text-start text-sm transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring max-sm:min-h-11"
                                    data-trevor-prepared={s.answer?.trim() ? "" : undefined}
                                >
                                    {s.question}
                                    {!s.answer?.trim() && !signedIn ? <span className="sr-only"> ({t("trevor.ask.signIn")})</span> : null}
                                </button>
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}
        </div>
    );
}

function TrevorAvatar(): React.ReactElement {
    return (
        <Avatar aria-hidden className="size-7 bg-primary font-semibold text-primary-foreground text-xs">
            <AvatarFallback className="bg-primary text-primary-foreground">T</AvatarFallback>
        </Avatar>
    );
}

function Turn({ turn, latest, storyId, signedIn, onJumpLine, onResume }: { turn: ThreadTurn; latest: boolean; storyId: string | null; signedIn: boolean; onJumpLine?: (line: number) => void; onResume: (turnId: number) => void }): React.ReactElement {
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
        const label = state.kind === "sending" ? t("trevor.ask.sending") : [t("trevor.ask.queued"), eta(state.etaSecs, t), state.position ? t("trevor.ask.position", { position: state.position }) : null].filter(Boolean).join(" · ");
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
    } else if (state.kind === "done") {
        const a = state.answer;
        const passages = a.source === "passages";
        reply = (
            <ChatMessage side="incoming" author={author} avatar={avatar} footer={<AnswerFooter answer={a} sourceId={sourceId} storyId={storyId} onJumpLine={onJumpLine} signedIn={signedIn} />} aria-label={t("trevor.answer.label")}>
                <div className="flex flex-col gap-2">
                    {passages ? (
                        <div className="flex flex-col gap-1">
                            <span className="self-start rounded-full bg-background/70 px-2 py-0.5 font-medium text-xs" data-trevor-passages>
                                {t("trevor.ask.passagesOnly")}
                            </span>
                            <p className="text-muted-foreground text-xs">{t("trevor.ask.passagesNote")}</p>
                        </div>
                    ) : null}
                    {a.text ? <AnswerText text={a.text} sourceId={sourceId} cited={new Set(a.citations.map((c) => c.n))} /> : null}
                </div>
            </ChatMessage>
        );
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
            <div aria-live="polite" aria-busy={pending} data-trevor-state={state.kind} data-trevor-live={latest ? "" : undefined}>
                {reply}
            </div>
        </div>
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
