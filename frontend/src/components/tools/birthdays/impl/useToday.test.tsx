import { act } from "react";
import { hydrateRoot } from "react-dom/client";
import { renderToString } from "react-dom/server";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";

import { useToday } from "./useToday";

function Today() {
    const today = useToday();
    return <b>{`${today.getMonth() + 1}/${today.getDate()}`}</b>;
}

describe("useToday", () => {
    const zone = process.env.TZ;
    beforeAll(() => {
        // 18:00 UTC on Oct 10 is 03:00 on Oct 11 in Tokyo.
        vi.useFakeTimers({ toFake: ["Date"] });
        vi.setSystemTime(Date.UTC(2026, 9, 10, 18));
        process.env.TZ = "Asia/Tokyo";
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    });
    afterAll(() => {
        vi.useRealTimers();
        if (zone === undefined) delete process.env.TZ;
        else process.env.TZ = zone;
    });

    it("renders the UTC day on the server", () => {
        expect(renderToString(<Today />)).toContain("10/10");
    });

    it("hydrates that day without a mismatch, then moves to the reader's own", async () => {
        const container = document.createElement("div");
        container.innerHTML = renderToString(<Today />);
        const errors: unknown[] = [];
        await act(async () => {
            hydrateRoot(container, <Today />, { onRecoverableError: (error) => errors.push(error) });
        });
        expect(errors).toEqual([]);
        expect(container.textContent).toBe("10/11");
    });
});
