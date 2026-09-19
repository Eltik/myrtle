import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { I18nProvider } from "#/lib/i18n/context";
import LanguageToggle from "./header/impl/LanguageToggle";
import { LocaleSwitcher } from "./LocaleSwitcher";

// `useLocaleSwitch` reads the current location to rebuild it under the new
// locale's basepath. Standing up a real router for that would test the router;
// the switch logic only needs a location to read.
vi.mock("@tanstack/react-router", () => ({
    useRouterState: <T,>({ select }: { select: (state: unknown) => T }): T => select({ location: { pathname: "/operators", searchStr: "" } }),
}));

const MESSAGES = {
    "common.localeSwitcher.language": "Language",
    "nav.languageToggle.trigger": "Language: {language}",
    "nav.languageToggle.triggerAria": "{label}. Choose a language.",
};

const ONE = [{ code: "en", nativeName: "English" }];
const TWO = [
    { code: "en", nativeName: "English" },
    { code: "ja", nativeName: "日本語" },
];

function mount(available: Array<{ code: string; nativeName: string }>, children: React.ReactNode) {
    return render(
        <I18nProvider locale="en" available={available} messages={MESSAGES}>
            {children}
        </I18nProvider>,
    );
}

// `globals: false`, so RTL's auto-cleanup is not installed.
afterEach(cleanup);

describe("LocaleSwitcher", () => {
    it("renders nothing when only one locale is enabled", () => {
        const { container } = mount(ONE, <LocaleSwitcher />);
        expect(container.firstChild).toBeNull();
    });

    it("lists every enabled locale under its own name, marking the active one", () => {
        mount(TWO, <LocaleSwitcher />);
        expect(screen.getAllByRole("button").map((b) => b.textContent)).toEqual(["English", "日本語"]);
        expect(screen.getByRole("button", { name: "English" }).getAttribute("aria-current")).toBe("true");
        expect(screen.getByRole("button", { name: "日本語" }).getAttribute("aria-current")).toBeNull();
    });
});

describe("LanguageToggle", () => {
    it("renders nothing when only one locale is enabled", () => {
        const { container } = mount(ONE, <LanguageToggle />);
        expect(container.firstChild).toBeNull();
    });

    it("shows the active language's code and names it for a screen reader", () => {
        mount(TWO, <LanguageToggle />);
        const trigger = screen.getByRole("button", { name: "Language: English. Choose a language." });
        expect(trigger.getAttribute("title")).toBe("Language: English");
        expect(trigger.textContent).toBe("en");
    });

    it("lists every enabled locale in the menu", async () => {
        mount(TWO, <LanguageToggle />);
        fireEvent.click(screen.getByRole("button", { name: /Choose a language/ }));
        const ja = await screen.findByText("\u65e5\u672c\u8a9e");
        expect(ja.closest("[lang]")?.getAttribute("lang")).toBe("ja");
    });
});
