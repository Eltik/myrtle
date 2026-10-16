import { describe, expect, it } from "vitest";
import type { AskResponse } from "#/types/generated/AskResponse";
import type { JobView } from "#/types/generated/JobView";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import type { AskEvent } from "./askState";
import { EMPTY_THREAD, type IThread, latestSpoiler, MAX_OPEN, openSlots, pendingCount, type ThreadAction, threadReducer } from "./thread";

const SHATTERPOINT = { eraId: "main_10", name: "Shatterpoint", position: 722 };
const accepted = (jobId: string, spoiler: AskResponse["spoilerSafeUpTo"] = SHATTERPOINT): AskResponse => ({ jobId, status: "queued", rung: "cf", spoilerSafeUpTo: spoiler, etaSecs: 20, streamUrl: `/api/trevor/jobs/${jobId}/stream` });
const answer = (text: string): TrevorAnswer => ({ id: `a-${text}`, version: "v", server: "en", eraPos: 722, rung: "cf", source: "live", answer: { answer: text, citations: [] } });
const done = (jobId: string, text: string): JobView => ({ jobId, status: "done", rung: "cf", position: 0, etaSecs: null, workerOnline: true, answer: answer(text), error: null });

function run(actions: ThreadAction[], start: IThread = EMPTY_THREAD): IThread {
    return actions.reduce((t, a) => threadReducer(t, a), start);
}
const ask = (question: string): ThreadAction => ({ type: "ask", question, token: null });
const ev = (id: number, event: AskEvent): ThreadAction => ({ type: "turn", id, event });
function kinds(t: IThread): string[] {
    return t.turns.map((x) => (x.kind === "ask" ? `${x.id}:${x.state.kind}` : `${x.id}:bank`));
}

describe("the ask thread", () => {
    it("sends each question at once, oldest first, each with its own id", () => {
        const t = run([ask("Who?"), ask("Why?")]);
        expect(kinds(t)).toEqual(["1:sending", "2:sending"]);
        expect(t.nextId).toBe(3);
        expect(openSlots(t)).toBe(2);
        expect(EMPTY_THREAD.turns).toEqual([]);
    });

    it("streams several turns at once and keeps them in the order asked when they finish out of order", () => {
        let t = run([ask("One?"), ask("Two?"), ask("Three?")]);
        t = run([ev(1, { type: "accepted", response: accepted("j1") }), ev(2, { type: "accepted", response: accepted("j2") }), ev(3, { type: "accepted", response: accepted("j3") })], t);
        t = run([ev(1, { type: "text", text: "Al" }), ev(2, { type: "text", text: "Be" }), ev(3, { type: "text", text: "Ga" })], t);
        expect(kinds(t)).toEqual(["1:streaming", "2:streaming", "3:streaming"]);
        t = run([ev(3, { type: "done", view: done("j3", "Gamma.") }), ev(1, { type: "text", text: "Alph" })], t);
        expect(kinds(t)).toEqual(["1:streaming", "2:streaming", "3:done"]);
        const first = t.turns[0];
        expect(first?.kind === "ask" && first.state.kind === "streaming" && first.state.text).toBe("Alph");
        t = run([ev(1, { type: "done", view: done("j1", "Alpha, in full.") })], t);
        const settled = t.turns[0];
        // The job's answer replaces the streamed text.
        expect(settled?.kind === "ask" && settled.state.kind === "done" && settled.state.answer.text).toBe("Alpha, in full.");
        expect(pendingCount(t)).toBe(1);
    });

    it("lets one turn fail without touching the others", () => {
        let t = run([ask("One?"), ask("Two?"), ev(1, { type: "accepted", response: accepted("j1") }), ev(2, { type: "accepted", response: accepted("j2") }), ev(2, { type: "text", text: "Half" })]);
        t = run([ev(1, { type: "jobFailed", message: "worker died" })], t);
        expect(kinds(t)).toEqual(["1:failed", "2:streaming"]);
        t = run([ev(2, { type: "done", view: done("j2", "Whole.") })], t);
        expect(kinds(t)).toEqual(["1:failed", "2:done"]);
    });

    it("holds a question past the open-job cap and sends the oldest held one when a slot frees", () => {
        let t = run([ask("1?"), ask("2?"), ask("3?"), ask("4?"), ask("5?")]);
        expect(MAX_OPEN).toBe(3);
        expect(kinds(t)).toEqual(["1:sending", "2:sending", "3:sending", "4:held", "5:held"]);
        t = run([ev(1, { type: "accepted", response: accepted("j1") }), ev(2, { type: "accepted", response: accepted("j2") }), ev(3, { type: "accepted", response: accepted("j3") })], t);
        expect(openSlots(t)).toBe(3);
        expect(kinds(t).slice(3)).toEqual(["4:held", "5:held"]);
        // Turn 2 finishes first: turn 4 (not 5) goes.
        t = run([ev(2, { type: "done", view: done("j2", "Two.") })], t);
        expect(kinds(t)).toEqual(["1:queued", "2:done", "3:queued", "4:sending", "5:held"]);
        // A refused send frees its slot too.
        t = run([ev(4, { type: "requestError", error: Object.assign(new Error("busy"), { status: 503 }) })], t);
        expect(kinds(t).slice(3)).toEqual(["4:refused", "5:sending"]);
        // A cached answer never held a job: it frees the slot it was sent in.
        t = run([ask("6?"), ev(5, { type: "accepted", response: { ...accepted("x"), jobId: null, status: "done", rung: "cache", answer: answer("Cached.") } })], t);
        expect(kinds(t).slice(4)).toEqual(["5:done", "6:sending"]);
    });

    it("counts twins that joined one job as one open job", () => {
        let t = run([ask("Same?"), ask("Same?"), ask("Other?"), ask("Fourth?")]);
        expect(kinds(t)[3]).toBe("4:held");
        // The backend joined turn 2 to turn 1's job: one job for two turns frees a slot, and turn 4 takes it.
        t = run([ev(1, { type: "accepted", response: accepted("j1") }), ev(2, { type: "accepted", response: accepted("j1") })], t);
        expect(kinds(t)).toEqual(["1:queued", "2:queued", "3:sending", "4:sending"]);
        expect(openSlots(t)).toBe(3);
        // Both twins settle from the same done view, each in its own bubble.
        t = run([ev(1, { type: "done", view: done("j1", "Shared.") }), ev(2, { type: "done", view: done("j1", "Shared.") })], t);
        expect(kinds(t).slice(0, 2)).toEqual(["1:done", "2:done"]);
    });

    it("keeps an offline job's slot until it is answered", () => {
        let t = run([ask("1?"), ask("2?"), ask("3?"), ask("4?")]);
        t = run([ev(1, { type: "accepted", response: accepted("j1") }), ev(1, { type: "offline" })], t);
        expect(kinds(t)).toEqual(["1:offline", "2:sending", "3:sending", "4:held"]);
        t = run([ev(1, { type: "resume" })], t);
        expect(kinds(t)[0]).toBe("1:queued");
    });

    it("puts a prepared answer after the questions before it, and ignores events for it", () => {
        const t = run([ask("Who?"), { type: "bank", suggestion: { id: "ask:1", question: "What is Afterglow?", answer: "A district." } }]);
        expect(t.turns[1]).toEqual({ id: 2, kind: "bank", suggestionId: "ask:1", question: "What is Afterglow?", answer: "A district." });
        expect(threadReducer(t, ev(2, { type: "offline" }))).toBe(t);
        expect(threadReducer(t, ev(9, { type: "offline" }))).toBe(t);
    });

    it("reports the newest spoiler horizon, and none before any ask reported one", () => {
        expect(latestSpoiler(EMPTY_THREAD)).toBeUndefined();
        const bankOnly = run([{ type: "bank", suggestion: { id: "a", question: "q", answer: "a" } }]);
        expect(latestSpoiler(bankOnly)).toBeUndefined();
        let t = run([ask("1?"), ev(2, { type: "accepted", response: accepted("j1") })], bankOnly);
        expect(latestSpoiler(t)).toEqual(SHATTERPOINT);
        t = run([ask("2?")], t);
        expect(latestSpoiler(t)).toEqual(SHATTERPOINT);
        t = run([ev(3, { type: "accepted", response: accepted("j2", null) })], t);
        expect(latestSpoiler(t)).toBeNull();
    });
});
