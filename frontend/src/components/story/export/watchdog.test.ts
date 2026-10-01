import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createWatchdog } from "./watchdog";

describe("createWatchdog", () => {
    beforeEach(() => vi.useFakeTimers());
    afterEach(() => vi.useRealTimers());

    it("fires once after the silence, not while kicked", () => {
        const stall = vi.fn();
        const dog = createWatchdog(60_000, stall);
        vi.advanceTimersByTime(59_000);
        dog.kick();
        vi.advanceTimersByTime(59_000);
        expect(stall).not.toHaveBeenCalled();
        vi.advanceTimersByTime(1_001);
        expect(stall).toHaveBeenCalledTimes(1);
        dog.kick();
        vi.advanceTimersByTime(120_000);
        expect(stall).toHaveBeenCalledTimes(1);
    });

    it("never fires once stopped", () => {
        const stall = vi.fn();
        const dog = createWatchdog(1_000, stall);
        dog.stop();
        vi.advanceTimersByTime(5_000);
        expect(stall).not.toHaveBeenCalled();
    });
});
