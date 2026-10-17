/**
 * The stories page's "Ask Trevor" tab (contract v1.1 section 6), mounted only
 * with `VITE_TREVOR=1`.
 *
 * The reader's chat (`AskChat`) with no story, so the spoiler horizon is the
 * reader's profile alone; the era it stops at is named above the composer.
 * The empty thread holds the era's popular questions (`GET /trevor/popular`),
 * each opening its stored answer with no request. "My questions"
 * (`GET /trevor/me/jobs`) sits beside the chat from 1024 px up and under it
 * below, and a past question opens into the thread, an open one followed again
 * until it lands. A backend without these routes (404) leaves the chat working
 * with neither list. Signed out, the tab explains and requests nothing.
 */
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { ChatMessage } from "#/components/chat/chat-message";
import { Button } from "#/components/ui/button";
import { useAuth } from "#/hooks/use-auth";
import { getMyJobs, trevorMyJobsQueryOptions, trevorPopularQueryOptions, trevorStatusQueryOptions } from "#/lib/api/trevor";
import { authActions } from "#/lib/auth/store";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { parseAnswer } from "#/lib/trevor/answer";
import { isFollowed } from "#/lib/trevor/askState";
import { type IMyJob, jobOpen, toTurnState } from "#/lib/trevor/library";
import { type IAskDeps, useTrevorThread } from "#/lib/trevor/useTrevorThread";
import { cn } from "#/lib/utils";
import type { TrevorPopularQuestion } from "#/types/generated/TrevorPopularQuestion";
import { AskChat } from "./AskChat";
import { MyQuestions } from "./MyQuestions";
import { LISTED_CHIP_CLASS } from "./QuestionChipRow";
import { TrevorAvatar } from "./TrevorAvatar";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

const MINE_LIMIT = 20;
/** Popular chips shown on a phone. */
const PHONE_CHIPS = 4;

export interface ILibraryAskProps {
    /** The tab is selected: status polling and end-following run only then. */
    active: boolean;
    /** Test seam for the ask hook's network. */
    askDeps?: IAskDeps;
}

export function LibraryAsk({ active, askDeps }: ILibraryAskProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const { user } = useAuth();
    const signedIn = user !== null;
    if (!signedIn) return <SignedOut />;
    return <SignedIn t={t} active={active} askDeps={askDeps} />;
}

function SignedIn({ t, active, askDeps }: { t: TrevorT; active: boolean; askDeps?: IAskDeps }): React.ReactElement {
    const queryClient = useQueryClient();
    const status = useQuery(trevorStatusQueryOptions(active));
    const popular = useQuery({ ...trevorPopularQueryOptions(true), enabled: active });
    const asker = useTrevorThread(null, askDeps);
    const [older, setOlder] = useState<IMyJob[]>([]);
    const [olderCursor, setOlderCursor] = useState<string | null | undefined>(undefined);
    const mine = useQuery({
        ...trevorMyJobsQueryOptions(MINE_LIMIT),
        enabled: active,
        // An open question in the list is followed until it lands: a deferred full answer shows up here.
        refetchInterval: (q) => (q.state.data?.jobs.some(jobOpen) ? 20_000 : false),
    });

    // A question accepted (it has a job now) or settling in the thread is news for My questions.
    const progress = asker.thread.turns.map((x) => (x.kind === "ask" ? `${x.id}:${x.state.kind}:${"jobId" in x.state ? x.state.jobId : ""}` : "")).join(",");
    const wasProgress = useRef(progress);
    useEffect(() => {
        if (progress !== wasProgress.current) void queryClient.invalidateQueries({ queryKey: ["trevor", "me", "jobs"] });
        wasProgress.current = progress;
    }, [progress, queryClient]);

    const following = new Set(asker.thread.turns.flatMap((x) => (x.kind === "ask" && isFollowed(x.state) ? [x.state.jobId] : [])));
    const openJob = (job: IMyJob) => {
        if (job.jobId && following.has(job.jobId)) return;
        asker.restore(toTurnState(job));
    };
    const openPopular = (q: TrevorPopularQuestion) => asker.restore({ kind: "done", question: q.question, answer: parseAnswer(q.answer), rung: q.answer.rung, spoiler: null, cached: true });
    const loadOlder = async () => {
        const cursor = olderCursor === undefined ? (mine.data?.nextBefore ?? null) : olderCursor;
        if (!cursor) return;
        const page = await getMyJobs(MINE_LIMIT, cursor).catch(() => null);
        if (!page) return;
        setOlder((prev) => [...prev, ...page.jobs]);
        setOlderCursor(page.nextBefore);
    };
    const jobs = [...(mine.data?.jobs ?? []), ...older];
    const moreCursor = olderCursor === undefined ? (mine.data?.nextBefore ?? null) : olderCursor;
    const myQuestions = mine.data === null ? null : <MyQuestions jobs={jobs} loading={mine.isPending} failed={mine.isError} hasMore={moreCursor !== null} onMore={() => void loadOlder()} onOpen={openJob} />;
    const questions = popular.data?.questions ?? [];
    const popularChips = questions.map((q) => ({ key: q.question, question: q.question, prepared: true, onPick: () => openPopular(q) }));

    return (
        <section aria-label={t("trevor.library.label")} className="grid gap-4 pt-4 lg:grid-cols-[minmax(0,1fr)_20rem]" data-trevor-library>
            <div className="flex h-[calc(100dvh-13rem)] max-h-[760px] min-h-[30rem] flex-col overflow-hidden rounded-xl border border-border bg-card">
                <AskChat
                    storyId={null}
                    panel={null}
                    signedIn
                    status={status.data}
                    thread={asker.thread}
                    onAsk={asker.ask}
                    onPrepared={asker.addPrepared}
                    onResume={asker.resume}
                    active={active}
                    spoilerFallback={popular.data?.spoiler}
                    moreQuestions={{ label: t("trevor.library.popular"), chips: popularChips }}
                    emptyState={
                        <div className="flex flex-col gap-5" data-trevor-library-empty>
                            <ChatMessage side="incoming" author="Trevor" avatar={<TrevorAvatar />}>
                                <p className="text-sm leading-relaxed">{t("trevor.library.greeting")}</p>
                                {popular.data?.spoiler === undefined ? <p className="mt-1 text-muted-foreground text-xs">{t("trevor.library.eraUnknown")}</p> : null}
                            </ChatMessage>
                            {questions.length > 0 ? (
                                <section className="flex flex-col gap-2" aria-labelledby="trevor-popular" data-trevor-popular>
                                    <h3 id="trevor-popular" className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                                        {t("trevor.library.popular")}
                                    </h3>
                                    <ul className="flex flex-wrap gap-2">
                                        {/* Under 640 px the first 4 only: the chat opens at its end, so a longer list would push the greeting out of view. */}
                                        {questions.map((q, i) => (
                                            <li key={q.question} className={cn("max-w-full", i >= PHONE_CHIPS && "max-sm:hidden")}>
                                                <button type="button" onClick={() => openPopular(q)} className={LISTED_CHIP_CLASS} data-trevor-prepared="">
                                                    {q.question}
                                                </button>
                                            </li>
                                        ))}
                                    </ul>
                                </section>
                            ) : null}
                        </div>
                    }
                />
            </div>
            {myQuestions ? <aside className="lg:max-h-[760px] lg:overflow-y-auto">{myQuestions}</aside> : null}
        </section>
    );
}

/** Signed out: what the tab does and why it needs an account. Nothing is requested. */
function SignedOut(): React.ReactElement {
    const t: TrevorT = useT("story");
    return (
        <section aria-label={t("trevor.library.label")} className="pt-4" data-trevor-library data-trevor-signed-out>
            <div className="mx-auto flex max-w-xl flex-col items-start gap-4 rounded-xl border border-border bg-card p-5 sm:p-6">
                <div className="flex items-center gap-3">
                    <TrevorAvatar />
                    <h2 className="font-semibold text-base">{t("trevor.library.signedOutTitle")}</h2>
                </div>
                <p className="text-sm leading-relaxed">{t("trevor.library.signedOut")}</p>
                <p className="text-muted-foreground text-sm">{t("trevor.library.signedOutPrepared")}</p>
                <Button className="max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                    {t("trevor.ask.signIn")}
                </Button>
            </div>
        </section>
    );
}
