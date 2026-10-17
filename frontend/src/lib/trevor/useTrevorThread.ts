/**
 * The Ask tab's hook: the thread, with every open question followed at once.
 *
 * `ask` adds a turn (`thread.ts` decides whether it is sent now or waits for
 * one of the backend's 3 open-job slots). Each turn being sent gets its own
 * request, posted with the story being read as `horizonStory` (the backend
 * lowers the spoiler horizon to it, never raises it); each turn with a job
 * gets its own `watchJob` (the stream, or polling every 5 s), so answers
 * stream into their own bubbles side by side and finish in any order.
 *
 * Requests and watchers are started by an effect from the thread itself, not
 * by the click: a turn that is queued or streaming and has no watcher gets
 * one. So a remount (StrictMode, a hot reload in the preview) that stopped
 * the watchers starts them again on the same jobs, keeping each turn's text,
 * instead of leaving an answer frozen mid-sentence. Unmounting stops every
 * watcher, so a closed reader leaves no request behind it.
 */
import { useCallback, useEffect, useReducer, useRef } from "react";
import * as api from "#/lib/api/trevor";
import type { AskResponse } from "#/types/generated/AskResponse";
import { type AskEvent, isFollowed } from "./askState";
import { EMPTY_THREAD, type IAskTurn, type IThread, MAX_OPEN, type TurnState, threadReducer } from "./thread";
import { createDeltaBuffer, type IDeltaBuffer, type IJobWatch, type IWatchDeps, watchJob } from "./watch";

export interface IAskDeps extends IWatchDeps {
    ask: (input: { question: string; horizonStory?: string | null; turnstileToken?: string | null }) => Promise<AskResponse>;
}

const DEFAULT_DEPS: IAskDeps = {
    ask: api.ask,
    openStream: api.streamJob,
    getJob: api.getJob,
    getStatus: api.getStatus,
};

interface ITrevorThread {
    thread: IThread;
    ask: (question: string, turnstileToken?: string | null) => void;
    addPrepared: (suggestion: { id: string; question: string; answer: string }) => void;
    /** Re-show a question asked earlier; one still open is followed again. */
    restore: (state: TurnState) => void;
    /** Follow an offline turn's job again. */
    resume: (turnId: number) => void;
}

export function useTrevorThread(storyId: string | null, deps: IAskDeps = DEFAULT_DEPS, cap: number = MAX_OPEN): ITrevorThread {
    const [thread, dispatch] = useReducer((t: IThread, a: Parameters<typeof threadReducer>[1]) => threadReducer(t, a, cap), EMPTY_THREAD);
    const depsRef = useRef(deps);
    depsRef.current = deps;
    const storyRef = useRef(storyId);
    storyRef.current = storyId;
    const watches = useRef(new Map<number, { jobId: string; watch: IJobWatch }>());
    const deltas = useRef(new Map<string, IDeltaBuffer>());
    const sent = useRef(new Set<number>());

    const turnEvent = useCallback((id: number, event: AskEvent) => dispatch({ type: "turn", id, event }), []);

    const send = useCallback(
        async (turn: IAskTurn) => {
            let response: AskResponse;
            try {
                response = await depsRef.current.ask({ question: turn.state.question, horizonStory: storyRef.current, turnstileToken: turn.token });
            } catch (error) {
                turnEvent(turn.id, { type: "requestError", error });
                return;
            }
            turnEvent(turn.id, { type: "accepted", response });
        },
        [turnEvent],
    );

    const follow = useCallback(
        (id: number, jobId: string) => {
            // Per turn AND job: a quick look's turn moves on to its deferred job, whose seqs start again.
            const key = `${id}:${jobId}`;
            let buffer = deltas.current.get(key);
            if (!buffer) {
                buffer = createDeltaBuffer();
                deltas.current.set(key, buffer);
            }
            const settle = (event: AskEvent) => {
                if (watches.current.get(id)?.jobId === jobId) watches.current.delete(id);
                turnEvent(id, event);
            };
            const watch = watchJob(
                jobId,
                {
                    onText: (text) => turnEvent(id, { type: "text", text }),
                    onProgress: (view) => turnEvent(id, { type: "progress", view }),
                    onDone: (view) => settle({ type: "done", view }),
                    onJobFailed: (message) => settle({ type: "jobFailed", message }),
                    onRequestError: (error) => settle({ type: "requestError", error }),
                    onOffline: () => settle({ type: "offline" }),
                },
                depsRef.current,
                buffer,
            );
            watches.current.set(id, { jobId, watch });
        },
        [turnEvent],
    );

    // Start what the thread asks for: a request for each turn being sent, a
    // watcher for each turn with a job in flight; stop a watcher whose turn settled.
    useEffect(() => {
        const live = new Set<number>();
        for (const turn of thread.turns) {
            if (turn.kind !== "ask") continue;
            const s = turn.state;
            if (s.kind === "sending" && !sent.current.has(turn.id)) {
                sent.current.add(turn.id);
                void send(turn);
            }
            if (isFollowed(s)) {
                live.add(turn.id);
                const w = watches.current.get(turn.id);
                if (w && w.jobId !== s.jobId) w.watch.stop();
                if (!w || w.jobId !== s.jobId) follow(turn.id, s.jobId);
            }
        }
        for (const [id, w] of watches.current) {
            if (live.has(id)) continue;
            w.watch.stop();
            watches.current.delete(id);
        }
    }, [thread, send, follow]);

    useEffect(() => {
        const map = watches.current;
        return () => {
            for (const w of map.values()) w.watch.stop();
            map.clear();
        };
    }, []);

    const ask = useCallback((question: string, turnstileToken: string | null = null) => dispatch({ type: "ask", question, token: turnstileToken }), []);
    const addPrepared = useCallback((suggestion: { id: string; question: string; answer: string }) => dispatch({ type: "bank", suggestion }), []);
    const resume = useCallback((turnId: number) => turnEvent(turnId, { type: "resume" }), [turnEvent]);
    const restore = useCallback((state: TurnState) => dispatch({ type: "restore", state }), []);

    return { thread, ask, addPrepared, resume, restore };
}
