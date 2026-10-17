/**
 * The Ask tab as a chat (coss chat kit: `components/chat/*`, `ui/composer`).
 *
 * The reader's questions are outgoing messages and Trevor's replies incoming
 * ones, in the order asked; the log follows the newest message while the
 * reader is at the bottom and offers "Jump to latest" otherwise. Each question
 * has its own job and watcher (`useTrevorThread`), so answers stream into their
 * own bubbles at once; past the backend's 3 open questions a turn waits, and
 * says so. When an answer settles and focus was lost, it returns to the
 * composer.
 *
 * The empty thread offers the panel's suggested questions: one with a prepared
 * answer adds it with no request, one without asks Trevor. Once the thread has
 * turns, the ones not yet asked stay above the composer (`QuestionChipRow`).
 * The stories page passes its own empty state, chips and era, and no story, so
 * the horizon is the reader's profile alone.
 *
 * A signed-in reader has a "Spoilers" switch beside the spoiler-safe label
 * (spoiler mode, contract v1.1 section 8, `SpoilerSwitch`); while it is on,
 * each question is sent with `spoilers: "all"` and the label says answers may
 * reveal later stories. `VITE_TREVOR_SPOILER_MODE=0` renders the label row as
 * it was before the switch.
 */
import { ArrowUpIcon } from "lucide-react";
import type React from "react";
import { useEffect, useId, useRef, useState } from "react";
import { ChatConversation } from "#/components/chat/chat-conversation";
import { ChatMessage } from "#/components/chat/chat-message";
import { useChatScroll } from "#/components/chat/hooks/use-chat-scroll";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { Button } from "#/components/ui/button";
import { Composer, ComposerActionGroup, ComposerActions, ComposerInput, ComposerSubmit, type ComposerValue, getComposerText } from "#/components/ui/composer";
import { useMediaQuery } from "#/hooks/use-media-query";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type IQuestionChip, unaskedChips } from "#/lib/trevor/chips";
import { trevorSpoilerModeEnabled, turnstileSiteKey } from "#/lib/trevor/flag";
import type { TrevorPanel, TrevorSuggestion } from "#/lib/trevor/panels";
import { useSpoilerMode } from "#/lib/trevor/spoilerMode";
import { type IThread, latestSpoiler, pendingCount } from "#/lib/trevor/thread";
import { cn } from "#/lib/utils";
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { SpoilerLabel } from "./AnswerView";
import { ChatTurn } from "./AskTurn";
import { chipRowEnabled, LISTED_CHIP_CLASS, QuestionChipRow } from "./QuestionChipRow";
import { SpoilerSwitch, SpoilersOnLabel } from "./SpoilerSwitch";
import { TrevorAvatar } from "./TrevorAvatar";
import { Turnstile } from "./Turnstile";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

/** The backend's `MAX_QUESTION_CHARS`. */
const QUESTION_MAX = 500;

export interface IAskChatProps {
    storyId: string | null;
    panel: TrevorPanel | null;
    signedIn: boolean;
    status: TrevorStatus | undefined;
    /** Every turn of the sitting, from `useTrevorThread`. */
    thread: IThread;
    /** `spoilers`: asked with spoiler mode on. */
    onAsk: (question: string, turnstileToken: string | null, spoilers: boolean) => void;
    onPrepared: (suggestion: { id: string; question: string; answer: string }) => void;
    onResume: (turnId: number) => void;
    onJumpLine?: (line: number) => void;
    /** The chat is on screen (sheet open, Ask tab selected): end-following runs only then. */
    active: boolean;
    /** Replaces the panel's greeting and suggestions in the empty thread. */
    emptyState?: React.ReactNode;
    /** The horizon to show before any ask has reported one (`undefined`: none known). */
    spoilerFallback?: TrevorSpoilerSafe | null;
    /** Replaces the panel's suggestions in the row above the composer once the thread has turns. */
    moreQuestions?: { label: string; chips: IQuestionChip[] };
}

export function AskChat({ storyId, panel, signedIn, status, thread, onAsk, onPrepared, onResume, onJumpLine, active, emptyState, spoilerFallback, moreQuestions }: IAskChatProps): React.ReactElement {
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
    const reported = latestSpoiler(thread);
    const spoiler = reported === undefined ? spoilerFallback : reported;
    const [spoilersOn, setSpoilersOn] = useSpoilerMode();
    const spoilerSwitch = signedIn && trevorSpoilerModeEnabled();
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
        onAsk(question, token, spoilersOn);
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

    const more = moreQuestions ?? {
        label: t("trevor.suggest.heading"),
        chips: (panel?.suggestions?.items ?? []).map((s) => ({ key: s.id, question: s.question, prepared: Boolean(s.answer?.trim()), onPick: () => onSuggestion(s) })),
    };
    // A chip's answer lands at the end of the thread: follow it there, as a typed question does.
    const rowChips = unaskedChips(more.chips, thread).map((chip) => ({
        ...chip,
        onPick: () => {
            scroll.atBottomRef.current = true;
            chip.onPick();
        },
    }));

    const text = getComposerText(draft);
    const tooLong = text.length > QUESTION_MAX;
    const blockedByCheck = siteKey !== null && token === null;

    return (
        <div ref={rootRef} className="flex min-h-0 flex-1 flex-col" data-trevor-chat>
            <ChatConversation scroll={scroll} label={t("trevor.chat.label")} jumpLabel={t("trevor.chat.jump")} live="off" contentClassName="px-4 pt-4 sm:px-6">
                {empty && emptyState ? (
                    emptyState
                ) : empty ? (
                    <EmptyChat panel={panel} signedIn={signedIn} onSuggestion={onSuggestion} />
                ) : (
                    <div className="flex flex-col gap-4">
                        {thread.turns.map((turn) => (
                            <ChatTurn key={turn.id} turn={turn} latest={turn.id === lastId} storyId={storyId} signedIn={signedIn} onJumpLine={onJumpLine} onResume={onResume} />
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
                {spoilerSwitch ? (
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1" data-trevor-spoiler-row>
                        {spoilersOn ? <SpoilersOnLabel /> : spoiler !== undefined ? <SpoilerLabel spoiler={spoiler} /> : null}
                        <SpoilerSwitch on={spoilersOn} onChange={setSpoilersOn} />
                    </div>
                ) : spoiler !== undefined ? (
                    <div className="flex">
                        <SpoilerLabel spoiler={spoiler} />
                    </div>
                ) : null}
                {!empty && chipRowEnabled() ? <QuestionChipRow label={more.label} chips={rowChips} /> : null}
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
                            <ComposerInput aria-label={t("trevor.ask.label")} aria-describedby={coarse ? undefined : hintId} placeholder={spoilersOn ? t("trevor.spoilers.placeholder") : t("trevor.ask.placeholder")} render={<textarea ref={inputRef} maxLength={QUESTION_MAX} data-trevor-question />} />
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
                                <button type="button" onClick={() => onSuggestion(s)} className={LISTED_CHIP_CLASS} data-trevor-prepared={s.answer?.trim() ? "" : undefined}>
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
