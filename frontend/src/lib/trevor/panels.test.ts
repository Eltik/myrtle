import { describe, expect, it } from "vitest";
import type { TrevorAnswer } from "#/types/generated/TrevorAnswer";
import sample from "./__fixtures__/panel_act18side_05_beg.json";
import { answerBlocks, answerInlines, parseAnswer } from "./answer";
import { ASK_IDLE, askBusy, askReducer } from "./askState";
import { panelCoverage, parsePanel, primerOf } from "./panels";

describe("parsePanel", () => {
    it("reads a published panel (LE-5, rank 776) whole", () => {
        const p = parsePanel(sample);
        expect(p).not.toBeNull();
        if (!p) return;
        expect(p.storyId).toBe("act18side_level_act18side_05_beg");
        expect(p.era).toEqual({ id: "main_10", name: "Shatterpoint", position: 722 });
        expect(p.recap?.soFar).toHaveLength(8);
        expect(p.whosHere?.characters.map((c) => c.name)).toEqual(["Ebenholz", "Czerny", "Hibiscus", "Lachmann", "Frivolous Noble", "Kreide", "Ursula"]);
        expect(p.terms?.items).toHaveLength(5);
        expect(p.suggestions?.items).toHaveLength(5);
        // Not a group's first story: a bank link, no primer text.
        expect(primerOf(p)).toBeNull();
        expect(panelCoverage(p)).toEqual({ recap: true, who: true, terms: true, suggestions: true });
    });

    it("drops a malformed item and keeps the rest", () => {
        const broken = structuredClone(sample) as unknown as { whosHere: { characters: unknown[] } };
        broken.whosHere.characters.splice(1, 0, { chunks: 3 }, 42);
        const p = parsePanel(broken);
        expect(p?.whosHere?.characters).toHaveLength(7);
    });

    it("refuses anything that is not a schema-1 panel", () => {
        expect(parsePanel(null)).toBeNull();
        expect(parsePanel({ ...sample, schema: 2 })).toBeNull();
        expect(parsePanel({ storyId: "x" })).toBeNull();
    });

    it("finds the primer on a group's first story", () => {
        const first = { ...sample, recap: { ...sample.recap, beforeYouStart: { primer: "  Margaret Nearl won the Major.  ", prior: ["act13side"] } } };
        expect(primerOf(parsePanel(first))).toBe("Margaret Nearl won the Major.");
    });
});

const answer = (over: Partial<TrevorAnswer> = {}): TrevorAnswer => ({
    id: "a1",
    version: "v",
    server: "en",
    eraPos: 722,
    rung: "cf",
    source: "live",
    answer: {
        answer: "**Hibiscus** is a nurse [2].\n\n*   She treats Infected [1, 2].\n*   She works in the Afterglow.\nOn this evidence, that is all.",
        citations: [{ n: 2, storyId: "s2", chunkId: "s2#0001", lineStart: 40, lineEnd: 52, text: "..." }, { n: 1, storyId: "s1", chunkId: "s1#0003", lineStart: 7, lineEnd: 9, text: "..." }, { n: "bad" }],
    },
    ...over,
});

describe("parseAnswer and the answer text", () => {
    it("reads the worker result, sorting citations and dropping malformed ones", () => {
        const a = parseAnswer(answer());
        expect(a.citations.map((c) => c.n)).toEqual([1, 2]);
        expect(a.citations[0]).toMatchObject({ storyId: "s1", lineStart: 7 });
        expect(a.source).toBe("live");
    });

    it("never throws on an empty body", () => {
        expect(parseAnswer(answer({ answer: null })).text).toBe("");
    });

    it("splits bold and multi-number citation markers", () => {
        expect(answerInlines("**A** b [1, 3] c [2]")).toEqual([
            { kind: "bold", text: "A" },
            { kind: "text", text: " b " },
            { kind: "cite", ns: [1, 3] },
            { kind: "text", text: " c " },
            { kind: "cite", ns: [2] },
        ]);
    });

    it("groups bullets into a list between paragraphs", () => {
        const blocks = answerBlocks(parseAnswer(answer()).text);
        expect(blocks.map((b) => b.kind)).toEqual(["p", "ul", "p"]);
        expect(blocks[1]?.kind === "ul" && blocks[1].items).toHaveLength(2);
    });
});

describe("the ask reducer", () => {
    const accepted = { jobId: "j1", status: "queued", rung: "cpu", spoilerSafeUpTo: { eraId: "main_10", name: "Shatterpoint", position: 722 }, etaSecs: 120, streamUrl: "/api/trevor/jobs/j1/stream" } as const;

    it("walks sending, queued, streaming, done", () => {
        let s = askReducer(ASK_IDLE, { type: "send", question: "Who is Hibiscus?" });
        expect(askBusy(s)).toBe(true);
        s = askReducer(s, { type: "accepted", response: accepted });
        expect(s).toMatchObject({ kind: "queued", jobId: "j1", etaSecs: 120, workerOnline: true });
        s = askReducer(s, { type: "text", text: "Hibis" });
        expect(s).toMatchObject({ kind: "streaming", text: "Hibis" });
        s = askReducer(s, { type: "done", view: { jobId: "j1", status: "done", rung: "cpu", position: 0, etaSecs: null, workerOnline: true, answer: answer(), error: null } });
        expect(s).toMatchObject({ kind: "done", question: "Who is Hibiscus?", cached: false, spoiler: { name: "Shatterpoint" } });
        expect(askBusy(s)).toBe(false);
    });

    it("takes a cached answer straight to done", () => {
        const s = askReducer(askReducer(ASK_IDLE, { type: "send", question: "q" }), { type: "accepted", response: { ...accepted, jobId: null, status: "done", rung: "cache", etaSecs: null, streamUrl: null, answer: answer({ source: "passages", rung: "passages" }) } });
        expect(s).toMatchObject({ kind: "done", cached: true, rung: "cache" });
        expect(s.kind === "done" && s.answer.source).toBe("passages");
    });

    it("turns a refused ask into its failure kind, and offline into a resumable state", () => {
        const refused = askReducer(askReducer(ASK_IDLE, { type: "send", question: "q" }), { type: "requestError", error: Object.assign(new Error("x"), { kind: "http", status: 429 }) });
        expect(refused).toMatchObject({ kind: "refused", failure: "rateLimited" });
        let s = askReducer(askReducer(ASK_IDLE, { type: "send", question: "q" }), { type: "accepted", response: accepted });
        s = askReducer(s, { type: "offline" });
        expect(s.kind).toBe("offline");
        s = askReducer(s, { type: "resume" });
        expect(s).toMatchObject({ kind: "queued", jobId: "j1" });
    });

    it("updates a queued job from a poll but never rewinds a streaming one", () => {
        let s = askReducer(askReducer(ASK_IDLE, { type: "send", question: "q" }), { type: "accepted", response: accepted });
        const poll = { jobId: "j1", status: "queued", rung: "cpu", position: 3, etaSecs: 90, workerOnline: false, answer: null, error: null } as const;
        s = askReducer(s, { type: "progress", view: poll });
        expect(s).toMatchObject({ kind: "queued", position: 3, etaSecs: 90, workerOnline: false });
        s = askReducer(s, { type: "text", text: "a" });
        expect(askReducer(s, { type: "progress", view: poll }).kind).toBe("streaming");
    });
});
