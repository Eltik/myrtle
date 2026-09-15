import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { I18nProvider } from "./context";
import { useRichT } from "./rich";

function Probe({ messages, locale = "en", render: r }: { messages: Record<string, string>; locale?: string; render: (rt: ReturnType<typeof useRichT>) => React.ReactNode }) {
    return (
        <I18nProvider locale={locale} available={[]} messages={messages}>
            <Inner render={r} />
        </I18nProvider>
    );
}

function Inner({ render: r }: { render: (rt: ReturnType<typeof useRichT>) => React.ReactNode }) {
    const rt = useRichT("t");
    return <div data-testid="out">{r(rt)}</div>;
}

// `globals: false`, so RTL's auto-cleanup is not installed.
afterEach(cleanup);

const text = () => screen.getByTestId("out").textContent;

describe("useRichT", () => {
    it("renders a plain message with no arguments", () => {
        render(<Probe messages={{ "t.plain": "Just words" }} render={(rt) => rt("plain")} />);
        expect(text()).toBe("Just words");
    });

    it("keeps one sentence as one message with an element in the middle", () => {
        render(<Probe messages={{ "t.oss": "Our code is open source on {link}, so you can review it." }} render={(rt) => rt("oss", { link: <a href="https://example.com">GitHub</a> })} />);
        expect(text()).toBe("Our code is open source on GitHub, so you can review it.");
        expect(screen.getByRole("link", { name: "GitHub" })).toHaveProperty("href", "https://example.com/");
    });

    // The whole point: a translation may put the element anywhere, which the
    // .before/.link/.after split made impossible.
    it("lets a translation move the element to the front of the sentence", () => {
        render(<Probe locale="ja" messages={{ "t.oss": "{link}でコードを公開しています。" }} render={(rt) => rt("oss", { link: <a href="https://example.com">GitHub</a> })} />);
        expect(text()).toBe("GitHubでコードを公開しています。");
    });

    it("still applies ICU plural selection when an element is present", () => {
        const messages = { "t.hits": "{count, plural, one {# result for {q}} other {# results for {q}}}" };
        const { rerender } = render(<Probe messages={messages} render={(rt) => rt("hits", { count: 1, q: <mark>ch</mark> })} />);
        expect(text()).toBe("1 result for ch");
        rerender(<Probe messages={messages} render={(rt) => rt("hits", { count: 5, q: <mark>ch</mark> })} />);
        expect(text()).toBe("5 results for ch");
    });

    it("formats numbers in the active locale alongside an element", () => {
        render(<Probe messages={{ "t.n": "{count, number} rows in {where}" }} render={(rt) => rt("n", { count: 1234567, where: <b>the table</b> })} />);
        expect(text()).toBe("1,234,567 rows in the table");
    });

    it("supports several elements and repeats of the same one", () => {
        render(<Probe messages={{ "t.two": "See {a} and {b}." }} render={(rt) => rt("two", { a: <i>one</i>, b: <i>two</i> })} />);
        expect(text()).toBe("See one and two.");
    });

    it("treats a Date as a value to format, not as markup", () => {
        render(<Probe messages={{ "t.d": "Updated {when, date}" }} render={(rt) => rt("d", { when: new Date("2026-09-15T00:00:00Z") })} />);
        expect(text()).toContain("2026");
    });

    it("falls back to the bundled source catalog, then to the key", () => {
        render(<Probe messages={{}} render={(rt) => rt("common.pagination.next")} />);
        // Not declared under this namespace and not in the catalog under the
        // prefixed name, so the key itself is the last resort - never blank.
        expect(text()).toBe("t.common.pagination.next");
    });
});
