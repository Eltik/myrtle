import { createContext, useContext, useMemo } from "react";

import type { Catalog } from "./catalog";
import { formatMessage, type MessageValues } from "./format";
import { DEFAULT_LOCALE, directionForLocale } from "./locale";
import { SOURCE_CATALOG } from "./source";

export interface II18nValue {
    locale: string;
    dir: "ltr" | "rtl";
    available: Array<{ code: string; nativeName: string }>;
    messages: Catalog;
    /**
     * The game-data server this locale reads operator/skill/stage text from
     * (`en`, `jp`, `kr`, `cn`, `tw`, `bili`). Read it with
     * {@link useGamedataServer} and pass it to the `lib/api` game-data
     * fetchers - it is part of their query keys.
     */
    gamedataServer: string;
}

const I18nContext = createContext<II18nValue>({
    locale: DEFAULT_LOCALE,
    dir: "ltr",
    available: [],
    messages: {},
    gamedataServer: "en",
});

export interface II18nProviderProps {
    locale: string;
    available: Array<{ code: string; nativeName: string }>;
    messages: Catalog;
    /** Defaults to `en`, which is the default-endpoint (unprefixed) server. */
    gamedataServer?: string;
    children: React.ReactNode;
}

export function I18nProvider({ locale, available, messages, gamedataServer = "en", children }: II18nProviderProps): React.ReactElement {
    const value = useMemo<II18nValue>(() => ({ locale, dir: directionForLocale(locale), available, messages, gamedataServer }), [locale, available, messages, gamedataServer]);
    return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): II18nValue {
    return useContext(I18nContext);
}

/**
 * Resolve one key. The order is the whole fallback story in three lines:
 * the locale's own translation, then the English source bundled with the
 * build.
 *
 * The database's `en` row reaches this through `messages` - so a copy fix
 * made in the admin panel wins over the bundled source without a rebuild -
 * and the bundled source is what keeps a brand-new or un-synced key from
 * rendering as a raw dotted identifier.
 */
function resolve(messages: Catalog, key: string): string {
    return messages[key] ?? SOURCE_CATALOG[key] ?? key;
}

export type TFunction = (key: string, values?: MessageValues) => string;

/**
 * `const t = useT("operators")` then `t("filters.empty")`.
 *
 * The namespace is a prefix, not a separate catalog: keys are globally unique
 * and namespaces exist to group work for translators and to bound what a route
 * needs to load. Passing a key that already contains a dot-path from the root
 * (`common.actions.save`) works from any namespace.
 */
export function useT(namespace?: string): TFunction {
    const { locale, messages } = useI18n();

    return useMemo(() => {
        const prefix = namespace ? `${namespace}.` : "";
        return (key: string, values?: MessageValues) => {
            const full = prefix && !key.startsWith(prefix) ? `${prefix}${key}` : key;
            const message = resolve(messages, full);
            return formatMessage(message, locale, values);
        };
    }, [locale, messages, namespace]);
}

/**
 * A formatter bound to the active locale, for the places that format numbers
 * and dates rather than messages. Everything in `lib/utils.ts` that used to
 * hardcode `en-US` takes its locale from here.
 */
export function useLocale(): string {
    return useI18n().locale;
}

/**
 * The Arknights client whose text backs the active locale's game data.
 *
 * Every `lib/api` game-data fetcher takes this as its `server` argument and
 * hits `/{server}/...` for anything other than the default, so that a locale
 * pinned to `jp` gets Japanese operator names rather than English ones under
 * translated chrome.
 */
export function useGamedataServer(): string {
    return useI18n().gamedataServer;
}
