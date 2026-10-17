import { describe, expect, it } from "vitest";
import type { AskResponse } from "#/types/generated/AskResponse";
import type { JobView } from "#/types/generated/JobView";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import { askReducer, isDeferred } from "./askState";
import { type IMyJob, jobOpen, parseMyJobs, parsePopular, timeOf, toTurnState } from "./library";
import { EMPTY_THREAD, latestSpoiler, openSlots, threadReducer } from "./thread";

const ERA = { eraId: "main_10", name: "Shatterpoint", position: 722 };
const stored = (text: string, source: "live" | "passages" = "live"): TrevorAnswer => ({
    id: `a-${text}`,
    version: "v",
    server: "en",
    eraPos: 722,
    rung: source === "live" ? "cpu" : "passages",
    source,
    answer: { answer: text, citations: [{ n: 1, storyId: "s1", chunkId: "c1", lineStart: 3, lineEnd: 5, text: "Quote." }] },
});
// The dev preview mock's shape: the passages answer inline, the ask's own job deferred.
const quickResponse = (extra: Partial<AskResponse> = {}): AskResponse => ({ jobId: "j1", status: "queued", rung: "passages", spoilerSafeUpTo: ERA, answer: stored("Top passages.", "passages"), etaSecs: 120, streamUrl: "/api/trevor/jobs/j1/stream", deferred: true, ...extra }) as AskResponse;
const view = (answer: TrevorAnswer | null, error: string | null = null): JobView => ({ jobId: "j1", status: answer ? "done" : "failed", rung: "cpu", position: 0, etaSecs: null, workerOnline: true, answer, error });

describe("quick look with a deferred full answer (contract v1.1 1b)", () => {
    it("finds the deferred job: the backend's deferredJobId, or the mock's flagged job", () => {
        expect(isDeferred(quickResponse())).toBe(true);
        expect(isDeferred(quickResponse({ deferred: false } as Partial<AskResponse>))).toBe(false);
        expect(isDeferred(quickResponse({ jobId: null }))).toBe(false);
        expect(isDeferred({ jobId: "p1", status: "queued", rung: "passages", spoilerSafeUpTo: ERA, etaSecs: 1, streamUrl: "", deferredJobId: "d1" } as AskResponse)).toBe(true);
        expect(isDeferred({ jobId: "j", status: "done", rung: "cache", spoilerSafeUpTo: ERA, answer: stored("A"), etaSecs: null, streamUrl: null })).toBe(false);
    });

    it("backend shape: follows the quick passages job, then the deferred full answer with the passages as the quick look", () => {
        const r = { jobId: "p1", status: "queued", rung: "passages", spoilerSafeUpTo: ERA, etaSecs: 1, streamUrl: "", deferredJobId: "d1" } as AskResponse;
        let s = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: r });
        expect(s.kind === "queued" && [s.jobId, s.next, s.quick]).toEqual(["p1", "d1", undefined]);
        s = askReducer(s, { type: "done", view: { ...view(stored("Passages.", "passages")), jobId: "p1" } });
        expect(s.kind === "queued" && [s.jobId, s.next, s.quick?.text]).toEqual(["d1", undefined, "Passages."]);
        s = askReducer(s, { type: "done", view: { ...view(stored("Full.")), jobId: "d1" } });
        expect(s.kind === "done" && [s.answer.text, s.quick ?? false]).toEqual(["Full.", false]);
        // A failed passages job still moves on to the full answer.
        const p = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: r });
        expect(askReducer(p, { type: "jobFailed", message: "x" })).toMatchObject({ kind: "queued", jobId: "d1" });
    });

    it("backend shape: an earlier era's cached answer is the quick look at once", () => {
        const look = stored("Earlier era.");
        const deferred = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: { jobId: "p1", status: "queued", rung: "passages", spoilerSafeUpTo: ERA, etaSecs: 1, streamUrl: "", deferredJobId: "d1", quickLook: look } as AskResponse });
        expect(deferred.kind === "queued" && [deferred.jobId, deferred.quick?.text]).toEqual(["d1", "Earlier era."]);
        const passagesOnly = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: { jobId: "p1", status: "queued", rung: "passages", spoilerSafeUpTo: ERA, etaSecs: 1, streamUrl: "", quickLook: look } as AskResponse });
        expect(passagesOnly.kind === "queued" && [passagesOnly.jobId, passagesOnly.quick?.text]).toEqual(["p1", "Earlier era."]);
    });

    it("shows the quick look at once and replaces it with the full answer, streaming in between", () => {
        let s = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: quickResponse() });
        expect(s.kind).toBe("queued");
        expect(s.kind === "queued" && s.quick?.source).toBe("passages");
        s = askReducer(s, { type: "text", text: "Writ" });
        expect(s.kind === "streaming" && s.quick?.citations.length).toBe(1);
        s = askReducer(s, { type: "done", view: view(stored("Written in full.")) });
        expect(s.kind === "done" && [s.answer.text, s.answer.source, s.quick ?? false]).toEqual(["Written in full.", "live", false]);
    });

    it("settles on the quick look, never an error, when the full answer fails", () => {
        const q = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: quickResponse() });
        for (const e of [
            { type: "done", view: view(null, "model died") },
            { type: "jobFailed", message: "x" },
            { type: "requestError", error: new Error("401") },
        ] as const) {
            const s = askReducer(q, e);
            expect(s.kind === "done" && [s.quick, s.fullFailed, s.answer.source]).toEqual([true, true, "passages"]);
        }
        const plain = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: { ...quickResponse(), answer: undefined } });
        expect(askReducer(plain, { type: "jobFailed", message: "x" }).kind).toBe("failed");
    });

    it("keeps the quick look through offline and resume", () => {
        const q = askReducer({ kind: "sending", question: "Who?" }, { type: "accepted", response: quickResponse() });
        const off = askReducer(q, { type: "offline" });
        const back = askReducer(off, { type: "resume" });
        expect(off.kind === "offline" && off.quick !== undefined).toBe(true);
        expect(back.kind === "queued" && back.quick !== undefined).toBe(true);
    });
});

describe("the stories tab's lists (contract v1.1 section 5)", () => {
    it("reads the popular questions and their era from the backend's {spoilerSafeUpTo, items}, deduping", () => {
        const p = parsePopular({ items: [{ question: " Who is Amiya? ", answer: stored("A."), askers: 4 }, { question: "who is amiya?" }, { nope: 1 }, "Why Chernobog?", { question: "Why Chernobog?", answer: null, askers: 2 }], spoilerSafeUpTo: ERA });
        expect(p.questions.map((q) => [q.question, q.answer !== null, q.askers])).toEqual([
            ["Who is Amiya?", true, 4],
            ["Why Chernobog?", false, 2],
        ]);
        expect(p.spoiler).toEqual(ERA);
        expect(parsePopular({ items: [], spoilerSafeUpTo: null })).toEqual({ questions: [], spoiler: null });
        // The old bare list is no longer read: it carries no era.
        expect(parsePopular([{ question: "Who is Amiya?", answer: null, askers: 2 }])).toEqual({ questions: [], spoiler: undefined });
        expect(parsePopular({ questions: [{ question: "Who is Amiya?" }], spoilerSafeUpTo: ERA }).questions).toEqual([]);
        expect(parsePopular("garbage")).toEqual({ questions: [], spoiler: undefined });
    });

    it("reads my jobs, dropping rows with no id or question", () => {
        const m = parseMyJobs({
            jobs: [
                { jobId: "j1", question: "Q1", status: "done", rung: "cpu", source: "live", createdAt: "2026-10-17T10:00:00Z", finishedAt: 1_792_000_000, answer: stored("A1") },
                { jobId: "j2", question: "Q2", status: "queued", rung: "cpu", deferred: true, createdAt: 1_792_000_000_000, quickLook: stored("P", "passages") },
                { question: "no id" },
                { jobId: "j3", question: "Q3", status: "weird" },
            ],
            nextBefore: "j3",
        });
        expect(m.jobs.map((j) => [j.jobId, j.status, j.deferred, j.answer !== null, j.quickLook !== null])).toEqual([
            ["j1", "done", false, true, false],
            ["j2", "queued", true, false, true],
            ["j3", "queued", false, false, false],
        ]);
        expect(m.nextBefore).toBe("j3");
        expect(m.jobs[0]?.createdAt).toBe(Date.parse("2026-10-17T10:00:00Z"));
        expect(m.jobs[0]?.finishedAt).toBe(1_792_000_000_000);
        expect(timeOf("not a date")).toBeNull();
        expect(parseMyJobs([{ id: "k", question: "bare" }]).jobs[0]?.jobId).toBe("k");
    });

    it("reads the backend's bare list: a cache hit has no job, and a full page pages before its last row", () => {
        const rows = [
            { jobId: null, question: "Cached?", status: "done", rung: "cache", source: "live", createdAt: "2026-10-17T10:00:00Z", finishedAt: null, answer: stored("C") },
            { jobId: "j2", question: "Open?", status: "queued", rung: "cpu", source: null, createdAt: "2026-10-17T09:00:00Z", finishedAt: null, answer: null },
        ];
        const m = parseMyJobs(rows, 2);
        expect(m.jobs.map((j) => [j.jobId, j.key])).toEqual([
            [null, "2026-10-17T10:00:00Z|Cached?"],
            ["j2", "j2"],
        ]);
        expect(m.nextBefore).toBe("2026-10-17T09:00:00Z");
        expect(parseMyJobs(rows, 20).nextBefore).toBeNull();
        expect(toTurnState(m.jobs[0] as IMyJob)).toMatchObject({ kind: "done", cached: true });
    });

    it("re-shows a past question as the turn it is: answered, open with its quick look, or failed", () => {
        const [done, open, failed] = parseMyJobs({
            jobs: [
                { jobId: "j1", question: "Q1", status: "done", rung: "cpu", answer: stored("A1") },
                { jobId: "j2", question: "Q2", status: "running", rung: "cpu", deferred: true, quickLook: stored("P", "passages") },
                { jobId: "j3", question: "Q3", status: "failed", error: "boom" },
            ],
        }).jobs;
        if (!done || !open || !failed) throw new Error("parse");
        expect(toTurnState(done)).toMatchObject({ kind: "done", cached: true });
        expect(toTurnState(open)).toMatchObject({ kind: "queued", jobId: "j2" });
        expect(toTurnState(failed)).toMatchObject({ kind: "failed", message: "boom" });
        expect([jobOpen(done), jobOpen(open), jobOpen(failed)]).toEqual([false, true, false]);
    });

    it("restores turns that do not move the spoiler pill and are followed when open", () => {
        let t = threadReducer(EMPTY_THREAD, { type: "ask", question: "Now?", token: null });
        t = threadReducer(t, { type: "turn", id: 1, event: { type: "accepted", response: { ...quickResponse(), answer: undefined } } });
        t = threadReducer(t, { type: "restore", state: { kind: "queued", question: "Old?", jobId: "j9", rung: "cpu", spoiler: null, etaSecs: null, position: null, workerOnline: true } });
        expect(latestSpoiler(t)).toEqual(ERA);
        expect(openSlots(t)).toBe(2);
        expect(threadReducer(t, { type: "restore", state: { kind: "sending", question: "x" } })).toBe(t);
    });
});
