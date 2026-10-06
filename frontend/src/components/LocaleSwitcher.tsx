import { useRouterState } from "@tanstack/react-router";
import { useCallback } from "react";

import { basepathForLocale, LOCALE_COOKIE, useI18n, useT } from "#/lib/i18n";
import type { IAvailableLocale } from "#/lib/i18n/catalog";
import { writePreferenceCookie } from "#/lib/i18n/cookie";
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
 * A locale's name with how much of the site it translates, for the menus that
 * have room for both. The percentage is formatted in the reader's locale, so
 * French reads "45 %".
 */
export function LocaleOptionLabel({ entry }: { entry: IAvailableLocale }): React.ReactElement {
    const { locale } = useI18n();
    return (
        <span className="flex items-baseline justify-between gap-3">
            <span className="truncate">{entry.nativeName}</span>
            {entry.completion !== undefined ? <span className="shrink-0 text-muted-foreground text-xs tabular-nums">{new Intl.NumberFormat(locale, { style: "percent" }).format(entry.completion)}</span> : null}
        </span>
    );
}

/**
 * The one place that knows how to change language: remember the choice, then leave the page for
 * the new locale's copy of the current URL.
 *
 * Shared by the footer switcher above, the header's globe menu and the Language section of the
 * appearance settings, so the cookie write and the full-document navigation are written once.
 * See {@link LocaleSwitcher} for why a client-side navigation is wrong here.
 *
 * Returns a callback rather than a component because the three call sites want three different
 * pieces of chrome around the same behaviour.
 */
export function useLocaleSwitch(): (code: string) => void {
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const search = useRouterState({ select: (s) => s.location.searchStr });

    return useCallback(
        (code: string) => {
            // Not awaited: the new URL carries the locale, so the next page does
            // not read the cookie. It only serves the bare root's redirect later.
            void writePreferenceCookie(LOCALE_COOKIE, code);

            const base = basepathForLocale(code);
            // `pathname` is already basepath-relative, so it composes directly.
            const next = base === "/" ? pathname : `${base}${pathname === "/" ? "" : pathname}`;
            window.location.assign(`${next}${search}`);
        },
        [pathname, search],
    );
}
