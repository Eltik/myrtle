import { describe, expect, it } from "vitest";
import type { AskState } from "./askState";
import { addBankTurn, EMPTY_THREAD, freezeLive, latestSpoiler, threadEmpty } from "./thread";

const SHATTERPOINT = { eraId: "main_10", name: "Shatterpoint", position: 722 };
const queued: AskState = { kind: "queued", question: "Who?", jobId: "j1", rung: "cpu", spoiler: SHATTERPOINT, etaSecs: 30, position: null, workerOnline: true };
const failed: AskState = { kind: "failed", question: "Why?", message: "" };

describe("the ask thread", () => {
    it("freezes nothing while the hook is idle", () => {
        expect(freezeLive(EMPTY_THREAD, { kind: "idle" })).toBe(EMPTY_THREAD);
        expect(threadEmpty(EMPTY_THREAD, { kind: "idle" })).toBe(true);
        expect(threadEmpty(EMPTY_THREAD, failed)).toBe(false);
    });

    it("freezes the question on screen with the next id, oldest first", () => {
        const one = freezeLive(EMPTY_THREAD, failed);
        const two = freezeLive(one, queued);
        expect(two.turns.map((t) => [t.id, t.kind === "ask" ? t.state.kind : t.kind])).toEqual([
            [1, "failed"],
            [2, "queued"],
        ]);
        expect(two.nextId).toBe(3);
        expect(EMPTY_THREAD.turns).toEqual([]);
    });

    it("puts a prepared answer after the question it interrupts", () => {
        const t = addBankTurn(EMPTY_THREAD, failed, { id: "ask:1", question: "What is Afterglow?", answer: "A district." });
        expect(t.turns.map((x) => x.kind)).toEqual(["ask", "bank"]);
        expect(t.turns[1]).toEqual({ id: 2, kind: "bank", suggestionId: "ask:1", question: "What is Afterglow?", answer: "A district." });
    });

    it("reports the newest spoiler horizon, and none before any ask reported one", () => {
        expect(latestSpoiler(EMPTY_THREAD, { kind: "idle" })).toBeUndefined();
        const bankOnly = addBankTurn(EMPTY_THREAD, { kind: "idle" }, { id: "a", question: "q", answer: "a" });
        expect(latestSpoiler(bankOnly, { kind: "idle" })).toBeUndefined();
        const withAsk = freezeLive(bankOnly, queued);
        expect(latestSpoiler(withAsk, { kind: "idle" })).toEqual(SHATTERPOINT);
        expect(latestSpoiler(withAsk, failed)).toEqual(SHATTERPOINT);
        const startOnly: AskState = { ...queued, spoiler: null };
        expect(latestSpoiler(withAsk, startOnly)).toBeNull();
    });
});
