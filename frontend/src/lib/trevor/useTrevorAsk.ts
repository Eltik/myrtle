/**
 * The ask box's hook: one question at a time, followed to its answer.
 *
 * `ask` posts the question with the story being read as `horizonStory` (the
 * backend lowers the spoiler horizon to it, never raises it), then follows the
 * job with `watchJob`: the stream, or polling every 5 s. Unmounting, a new
 * question and `reset` all stop the watch, so a closed panel leaves no request
 * behind it.
 */
import { useCallback, useEffect, useReducer, useRef } from "react";
import * as api from "#/lib/api/trevor";
import type { AskResponse } from "#/types/generated/AskResponse";
import { ASK_IDLE, type AskState, askReducer } from "./askState";
import { type IJobWatch, type IWatchDeps, watchJob } from "./watch";

export interface IAskDeps extends IWatchDeps {
    ask: (input: { question: string; horizonStory?: string | null; turnstileToken?: string | null }) => Promise<AskResponse>;
}

const DEFAULT_DEPS: IAskDeps = {
    ask: api.ask,
    openStream: api.streamJob,
    getJob: api.getJob,
    getStatus: api.getStatus,
};

export function useTrevorAsk(storyId: string | null, deps: IAskDeps = DEFAULT_DEPS): { state: AskState; ask: (question: string, turnstileToken?: string | null) => Promise<void>; resume: () => void; reset: () => void } {
    const [state, dispatch] = useReducer(askReducer, ASK_IDLE);
    const watch = useRef<IJobWatch | null>(null);
    const depsRef = useRef(deps);
    depsRef.current = deps;
    const stateRef = useRef(state);
    stateRef.current = state;

    const stopWatch = useCallback(() => {
        watch.current?.stop();
        watch.current = null;
    }, []);
    useEffect(() => stopWatch, [stopWatch]);

    const follow = useCallback(
        (jobId: string) => {
            stopWatch();
            watch.current = watchJob(
                jobId,
                {
                    onText: (text) => dispatch({ type: "text", text }),
                    onProgress: (view) => dispatch({ type: "progress", view }),
                    onDone: (view) => dispatch({ type: "done", view }),
                    onJobFailed: (message) => dispatch({ type: "jobFailed", message }),
                    onRequestError: (error) => dispatch({ type: "requestError", error }),
                    onOffline: () => dispatch({ type: "offline" }),
                },
                depsRef.current,
            );
        },
        [stopWatch],
    );

    const ask = useCallback(
        async (question: string, turnstileToken?: string | null) => {
            stopWatch();
            dispatch({ type: "send", question });
            let response: AskResponse;
            try {
                response = await depsRef.current.ask({ question, horizonStory: storyId, turnstileToken });
            } catch (error) {
                dispatch({ type: "requestError", error });
                return;
            }
            dispatch({ type: "accepted", response });
            if (!response.answer && response.jobId) follow(response.jobId);
        },
        [follow, stopWatch, storyId],
    );

    const resume = useCallback(() => {
        const s = stateRef.current;
        if (s.kind !== "offline") return;
        dispatch({ type: "resume" });
        follow(s.jobId);
    }, [follow]);

    const reset = useCallback(() => {
        stopWatch();
        dispatch({ type: "reset" });
    }, [stopWatch]);

    return { state, ask, resume, reset };
}
