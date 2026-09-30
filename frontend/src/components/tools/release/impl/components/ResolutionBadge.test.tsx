import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { Resolution } from "#/types/generated/Resolution";
import { messages as helperMessages } from "../helpers.messages";
import { dueNow } from "../resolution";
import { ResolutionBadge } from "./ResolutionBadge";
import { messages } from "./ResolutionBadge.messages";

const MESSAGES = Object.fromEntries(Object.entries({ ...messages, ...helperMessages }).map(([key, m]) => [`tools.${key}`, m.text]));

const TODAY = new Date(2026, 8, 30, 12);
const SEP_28 = Math.floor(new Date(2026, 8, 28, 19, 30).getTime() / 1000);

function mount(resolution: Resolution) {
    return render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={MESSAGES}>
            <ResolutionBadge resolution={resolution} today={TODAY} />
        </I18nProvider>,
    );
}

afterEach(cleanup);

describe("ResolutionBadge", () => {
    it("reads an overdue estimate as not yet in EN, with the date it was due", () => {
        const { container } = mount(dueNow({ status: "estimated", enStart: SEP_28, lo: SEP_28, hi: SEP_28 }, TODAY));
        expect(screen.getByText(messages["release.badge.overdue"].text)).toBeTruthy();
        expect(container.querySelector(`[title="${messages["release.badge.overdue.title"].text}"]`)).toBeTruthy();
        expect(container.textContent).toContain("est. Sep 28, 2026");
        expect(container.textContent).toContain("2 days ago");
        expect(container.textContent).not.toContain("yesterday");
    });

    it("reads an unlisted sale as not in EN's shop, with no date", () => {
        const { container } = mount({ status: "unlisted" });
        const badge = screen.getByText(messages["release.badge.unlisted"].text);
        expect(badge.getAttribute("title")).toBe(messages["release.badge.unlisted.title"].text);
        expect(container.textContent).toBe(messages["release.badge.unlisted"].text);
    });

    it("keeps a future estimate as it was", () => {
        const later = SEP_28 + 30 * 86_400;
        mount({ status: "estimated", enStart: later, lo: later, hi: later });
        expect(screen.getByText(messages["release.badge.estimated"].text)).toBeTruthy();
        expect(screen.queryByText(messages["release.badge.overdue"].text)).toBeNull();
    });
});
