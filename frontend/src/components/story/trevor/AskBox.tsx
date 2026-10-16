/**
 * THE ASK BOX: a question, then its life from queued to answered.
 *
 * Signed out, it is a sign-in prompt (asking needs an account; the panels do
 * not). Signed in, it is a textarea capped at the backend's 500 characters,
 * the Turnstile widget when a site key is configured, and one live region
 * below that announces each state: sending, queued (with the ETA and place in
 * line), writing (the text as it streams, `aria-busy` so a screen reader reads
 * it once it is whole), the answer, or the refusal in words.
 */
import { SendHorizontalIcon } from "lucide-react";
import type React from "react";
import { useId, useState } from "react";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Spinner } from "#/components/ui/spinner";
import { Textarea } from "#/components/ui/textarea";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type AskState, askBusy } from "#/lib/trevor/askState";
import type { TrevorFailureKind } from "#/lib/trevor/errors";
import { turnstileSiteKey } from "#/lib/trevor/flag";
import type { TrevorStatus } from "#/types/generated/TrevorStatus";
import { AnswerText } from "./AnswerText";
import { AnswerView, SpoilerLabel } from "./AnswerView";
import { Turnstile } from "./Turnstile";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

/** The backend's `MAX_QUESTION_CHARS`. */
export const QUESTION_MAX = 500;

export interface IAskBoxProps {
    storyId: string | null;
    signedIn: boolean;
    status: TrevorStatus | undefined;
    state: AskState;
    onAsk: (question: string, turnstileToken: string | null) => void;
    onResume: () => void;
    onReset: () => void;
    onJumpLine?: (line: number) => void;
}

export function AskBox({ storyId, signedIn, status, state, onAsk, onResume, onReset, onJumpLine }: IAskBoxProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const inputId = useId();
    const hintId = useId();
    const [question, setQuestion] = useState("");
    const siteKey = turnstileSiteKey();
    const [token, setToken] = useState<string | null>(null);
    const [resetKey, setResetKey] = useState(0);
    const busy = askBusy(state);
    const trimmed = question.trim();
    const canSend = signedIn && !busy && trimmed !== "" && question.length <= QUESTION_MAX && (siteKey === null || token !== null);

    const submit = () => {
        if (!canSend) return;
        onAsk(trimmed, token);
        setQuestion("");
        if (siteKey) setResetKey((k) => k + 1);
    };

    if (!signedIn) {
        return (
            <div className="flex flex-col items-start gap-3 rounded-lg border border-border border-dashed p-4" data-trevor-signed-out>
                <p className="text-muted-foreground text-sm">{t("trevor.ask.signedOut")}</p>
                <Button size="sm" className="max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                    {t("trevor.ask.signIn")}
                </Button>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-3" data-trevor-ask>
            {status && !status.online ? (
                <Alert variant="warning">
                    <AlertDescription>{t("trevor.ask.statusOffline")}</AlertDescription>
                </Alert>
            ) : null}
            <form
                className="flex flex-col gap-2"
                onSubmit={(e) => {
                    e.preventDefault();
                    submit();
                }}
            >
                <label htmlFor={inputId} className="sr-only">
                    {t("trevor.ask.label")}
                </label>
                <Textarea
                    id={inputId}
                    value={question}
                    maxLength={QUESTION_MAX}
                    rows={3}
                    aria-describedby={hintId}
                    placeholder={t("trevor.ask.placeholder")}
                    onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setQuestion(e.target.value)}
                    onKeyDown={(e: React.KeyboardEvent<HTMLTextAreaElement>) => {
                        // Enter sends, Shift+Enter is a new line; an IME composing a word keeps its Enter.
                        if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                            e.preventDefault();
                            submit();
                        }
                    }}
                    data-trevor-question
                />
                <div className="flex items-start justify-between gap-3">
                    <p id={hintId} className="text-muted-foreground text-xs">
                        {t("trevor.ask.hint")}
                    </p>
                    <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums" aria-hidden>
                        {t("trevor.ask.count", { count: question.length, max: QUESTION_MAX })}
                    </span>
                </div>
                {siteKey ? <Turnstile siteKey={siteKey} onToken={setToken} resetKey={resetKey} /> : null}
                <Button type="submit" size="sm" className="self-end max-sm:h-11" disabled={!canSend} loading={state.kind === "sending"}>
                    <SendHorizontalIcon />
                    {t("trevor.ask.submit")}
                </Button>
            </form>
            <AskResult storyId={storyId} signedIn={signedIn} state={state} onResume={onResume} onReset={onReset} onJumpLine={onJumpLine} />
        </div>
    );
}

/** The live region under the box: every state of the current question. */
export function AskResult({ storyId, signedIn, state, onResume, onReset, onJumpLine }: Pick<IAskBoxProps, "storyId" | "signedIn" | "state" | "onResume" | "onReset" | "onJumpLine">): React.ReactElement {
    const t: TrevorT = useT("story");
    const describe = useErrorMessage();
    const streaming = state.kind === "streaming";
    return (
        <div aria-live="polite" aria-busy={streaming || state.kind === "sending"} className="flex flex-col gap-3" data-trevor-state={state.kind}>
            {state.kind === "idle" ? null : (
                <p className="border-primary/40 border-s-2 ps-3 font-medium text-sm" data-trevor-asked>
                    {state.question}
                </p>
            )}
            {state.kind === "sending" ? <StatusLine spinner text={t("trevor.ask.sending")} /> : null}
            {state.kind === "queued" ? (
                <div className="flex flex-col gap-2">
                    <StatusLine spinner text={[t("trevor.ask.queued"), eta(state.etaSecs, t), state.position ? t("trevor.ask.position", { position: state.position }) : null].filter(Boolean).join(" · ")} />
                    {!state.workerOnline ? <p className="text-muted-foreground text-xs">{t("trevor.ask.noWorker")}</p> : null}
                    <SpoilerLabel spoiler={state.spoiler} />
                </div>
            ) : null}
            {streaming ? (
                <div className="flex flex-col gap-2">
                    <StatusLine spinner text={t("trevor.ask.writing")} />
                    <AnswerText text={state.text} sourceId={null} cited={new Set()} />
                </div>
            ) : null}
            {state.kind === "offline" ? (
                <div className="flex flex-col items-start gap-2">
                    <p className="text-muted-foreground text-sm">{t("trevor.ask.offline")}</p>
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={onResume}>
                        {t("trevor.ask.checkAgain")}
                    </Button>
                </div>
            ) : null}
            {state.kind === "done" ? <AnswerView answer={state.answer} spoiler={state.spoiler} storyId={storyId} onJumpLine={onJumpLine} signedIn={signedIn} /> : null}
            {state.kind === "failed" ? <p className="text-sm">{t("trevor.ask.failed")}</p> : null}
            {state.kind === "refused" ? (
                <div className="flex flex-col items-start gap-2">
                    <p className="text-sm" data-trevor-refused={state.failure}>
                        {refusal(state.failure, t) ?? describe(state.error)}
                    </p>
                    {state.failure === "signedOut" ? (
                        <Button size="sm" className="max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                            {t("trevor.ask.signIn")}
                        </Button>
                    ) : null}
                </div>
            ) : null}
            {state.kind === "done" || state.kind === "failed" || state.kind === "refused" ? (
                <Button variant="ghost" size="sm" className="self-start max-sm:h-11" onClick={onReset}>
                    {t("trevor.ask.another")}
                </Button>
            ) : null}
        </div>
    );
}

function StatusLine({ text, spinner }: { text: string; spinner?: boolean }): React.ReactElement {
    return (
        <p className="flex items-center gap-2 text-muted-foreground text-sm">
            {spinner ? <Spinner className="size-4" aria-hidden /> : null}
            {text}
        </p>
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
