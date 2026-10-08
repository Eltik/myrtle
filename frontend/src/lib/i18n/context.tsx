import { createContext, useContext, useMemo } from "react";

import type { Catalog, IAvailableLocale } from "./catalog";
import { formatMessage, type MessageValues } from "./format";
import { DEFAULT_LOCALE, directionForLocale } from "./locale";
import { SOURCE_CATALOG } from "./source";

export interface II18nValue {
    locale: string;
    dir: "ltr" | "rtl";
    available: IAvailableLocale[];
    messages: Catalog;
    /**
     * The game-data server this render reads operator/skill/stage text from
     * (`en`, `jp`, `kr`, `cn`, `tw`, `bili`): the visitor's pick, else the
     * locale's. Read it with
     * {@link useGamedataServer} and pass it to the `lib/api` game-data
     * fetchers - it is part of their query keys.
     */
    gamedataServer: string;
    /** The server this locale reads when the visitor has not picked one. */
    localeGamedataServer: string;
    /** Whether {@link gamedataServer} is the visitor's own pick. */
    gamedataServerPicked: boolean;
    /** The servers the backend has loaded, the default first. */
    gamedataServers: string[];
}

const I18nContext = createContext<II18nValue>({
    locale: DEFAULT_LOCALE,
    dir: "ltr",
    available: [],
    messages: {},
    gamedataServer: "en",
    localeGamedataServer: "en",
    gamedataServerPicked: false,
    gamedataServers: [],
});

export interface II18nProviderProps {
    locale: string;
    available: IAvailableLocale[];
    messages: Catalog;
    /** Defaults to `en`, which is the default-endpoint (unprefixed) server. */
    gamedataServer?: string;
    /** Defaults to {@link gamedataServer}. */
    localeGamedataServer?: string;
    gamedataServerPicked?: boolean;
    gamedataServers?: string[];
    children: React.ReactNode;
}

/** One shared empty default, so an omitted prop does not churn the memo below. */
const NO_SERVERS: string[] = [];

export function I18nProvider({ locale, available, messages, gamedataServer = "en", localeGamedataServer = gamedataServer, gamedataServerPicked = false, gamedataServers = NO_SERVERS, children }: II18nProviderProps): React.ReactElement {
    const value = useMemo<II18nValue>(() => ({ locale, dir: directionForLocale(locale), available, messages, gamedataServer, localeGamedataServer, gamedataServerPicked, gamedataServers }), [locale, available, messages, gamedataServer, localeGamedataServer, gamedataServerPicked, gamedataServers]);
    return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): II18nValue {
    return useContext(I18nContext);
}

/**
 * Resolve one key: the locale's own translation, then the English source
 * bundled with the build.
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
 * and dates rather than messages. The formatters in `lib/utils.ts` take their
 * locale from here instead of hardcoding `en-US`.
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
