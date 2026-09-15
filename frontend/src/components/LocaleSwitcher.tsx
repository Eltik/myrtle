import { useRouterState } from "@tanstack/react-router";
import { useCallback } from "react";

import { basepathForLocale, LOCALE_COOKIE, useI18n, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./LocaleSwitcher.messages";

/**
 * Language switcher.
 *
 * Renders nothing when only one locale is enabled, so it costs the current
 * single-locale deployment no pixels and no decisions.
 *
 * Switching locale is a full document navigation rather than a client-side
 * one, and deliberately so: the locale lives in the router's `basepath`, which
 * is fixed when the router is constructed. A client-side navigation would keep
 * the old basepath and start building wrong URLs. A real navigation also means
 * the server re-renders in the new language, so the first paint after the
 * switch is already translated.
 */
export function LocaleSwitcher({ className }: { className?: string }): React.ReactElement | null {
    const { locale, available } = useI18n();
    const t: TypedT<typeof messages> = useT("common");
    const go = useLocaleSwitch();

    if (available.length < 2) return null;

    return (
        <fieldset className={cn("inline-flex items-center gap-1.5 border-0 p-0", className)}>
            <legend className="sr-only">{t("localeSwitcher.language")}</legend>
            <div className="inline-flex items-center gap-1">
                {available.map((entry) => (
                    <button
                        key={entry.code}
                        type="button"
                        lang={entry.code}
                        aria-current={entry.code === locale ? "true" : undefined}
                        onClick={() => entry.code !== locale && go(entry.code)}
                        className={entry.code === locale ? "rounded px-1.5 py-0.5 font-medium font-sans text-[12.5px] text-foreground leading-none" : "rounded px-1.5 py-0.5 font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:text-foreground"}
                    >
                        {entry.nativeName}
                    </button>
                ))}
            </div>
        </fieldset>
    );
}

/**
 * The one place that knows how to change language: remember the choice, then
 * leave the page for the new locale's copy of the current URL.
 *
 * Shared by the three controls that offer the choice - the footer switcher
 * above, the header's globe menu, and the Language section of the appearance
 * settings - so that the cookie write and the full-document navigation are
 * written once. The navigation is not an accident of the first implementation:
 * see the note on {@link LocaleSwitcher} for why a client-side one is wrong
 * here.
 *
 * Returns a callback rather than a component because the three call sites want
 * three different pieces of chrome around the same behaviour.
 */
export function useLocaleSwitch(): (code: string) => void {
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const search = useRouterState({ select: (s) => s.location.searchStr });

    return useCallback(
        (code: string) => {
            rememberLocale(code);

            const base = basepathForLocale(code);
            // `pathname` is already basepath-relative, so it composes directly.
            const next = base === "/" ? pathname : `${base}${pathname === "/" ? "" : pathname}`;
            window.location.assign(`${next}${search}`);
        },
        [pathname, search],
    );
}

/**
 * Persist the chosen language so the bare root can send this visitor straight
 * back to it next time.
 *
 * `CookieStore` where the browser has it, `document.cookie` otherwise -
 * `CookieStore` is Chromium-only today, so the fallback is the path Safari and
 * Firefox actually take, not dead code. `SameSite=Lax` because the cookie is
 * read on a top-level navigation; a year because a language preference does
 * not go stale.
 */
function rememberLocale(code: string): void {
    const oneYear = 31_536_000;
    try {
        // `CookieStore` is Chromium-only today, so the `document.cookie`
        // fallback is the path Safari and Firefox actually take, not dead code.
        if (typeof cookieStore !== "undefined") {
            void cookieStore.set({ name: LOCALE_COOKIE, value: code, path: "/", expires: Date.now() + oneYear * 1000, sameSite: "lax" });
            return;
        }
        // biome-ignore lint/suspicious/noDocumentCookie: CookieStore is unavailable in this branch by construction - that is what the guard above tests.
        document.cookie = `${LOCALE_COOKIE}=${encodeURIComponent(code)}; path=/; max-age=${oneYear}; samesite=lax`;
    } catch {
        // A blocked cookie only costs the redirect on the next bare visit.
    }
}
