import { act, type ReactNode } from "react";
import { hydrateRoot } from "react-dom/client";
import { renderToString } from "react-dom/server";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { I18nProvider } from "./context";
import { useFormatters } from "./formatters";

// 20:00 UTC on May 29 is already May 30 in Tokyo: the split that made the
// server HTML and the hydrating render disagree.
const AT = Date.UTC(2022, 4, 29, 20);
const OPTIONS: Intl.DateTimeFormatOptions = { month: "short", day: "numeric", year: "numeric" };

function Registered() {
    const f = useFormatters();
    return <b>{f.date(AT, OPTIONS)}</b>;
}

function Ago() {
    const f = useFormatters();
    return <b>{f.relative(new Date(AT).toISOString())}</b>;
}

function Day() {
    const f = useFormatters();
    return <b>{f.calendarDay(new Date(2022, 4, 30), OPTIONS)}</b>;
}

const tree = (node: ReactNode = <Registered />) => (
    <I18nProvider locale="en" available={[]} messages={{}}>
        {node}
    </I18nProvider>
);

describe("useFormatters().date across hydration", () => {
    const zone = process.env.TZ;
    beforeAll(() => {
        process.env.TZ = "Asia/Tokyo";
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    });
    afterAll(() => {
        if (zone === undefined) delete process.env.TZ;
        else process.env.TZ = zone;
    });

    it("prints UTC on the server, whatever the server's own zone", () => {
        expect(renderToString(tree())).toContain("May 29, 2022");
    });

    it("hydrates a server in another zone without a mismatch, then prints the browser's zone", async () => {
        const container = document.createElement("div");
        // Server on US time (still May 29 there), browser in Tokyo.
        process.env.TZ = "America/Los_Angeles";
        container.innerHTML = renderToString(tree());
        process.env.TZ = "Asia/Tokyo";
        const errors: unknown[] = [];
        await act(async () => {
            hydrateRoot(container, tree(), { onRecoverableError: (error) => errors.push(error) });
        });
        expect(errors).toEqual([]);
        expect(container.textContent).toBe("May 30, 2022");
    });

    it("keeps a calendar day built from local parts on its own day", () => {
        // Tokyo's local midnight on May 30 is still May 29 in UTC.
        expect(renderToString(tree(<Day />))).toContain("May 30, 2022");
    });

    it("prints a relative time as its UTC date until hydration, then relative", async () => {
        const container = document.createElement("div");
        container.innerHTML = renderToString(tree(<Ago />));
        expect(container.textContent).toBe("May 29, 2022");
        const errors: unknown[] = [];
        await act(async () => {
            hydrateRoot(container, tree(<Ago />), { onRecoverableError: (error) => errors.push(error) });
        });
        expect(errors).toEqual([]);
        expect(container.textContent).toMatch(/ago$/);
    });
});
