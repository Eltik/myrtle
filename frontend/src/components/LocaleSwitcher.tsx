import { useRouterState } from "@tanstack/react-router";
import { useCallback } from "react";

import { basepathForLocale, LOCALE_COOKIE, useI18n } from "#/lib/i18n";
import type { IAvailableLocale } from "#/lib/i18n/catalog";
import { writePreferenceCookie } from "#/lib/i18n/cookie";

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
 * Shared by the header's language menu, the mobile drawer and the Language card of the
 * appearance settings, so the cookie write and the navigation are written once. Returns a
 * callback rather than a component because the three call sites want three different pieces of
 * chrome around the same behaviour.
 *
 * A full document navigation rather than a client-side one, deliberately: the router learns its
 * locale from the first URL it parses (`router.tsx`) and the client memoizes the catalog it
 * hydrated with (`lib/root-context.ts`), so a client-side switch would keep the old language
 * while building new-locale URLs. A real navigation also means the server renders the new
 * language, so the first paint after the switch is already translated.
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
