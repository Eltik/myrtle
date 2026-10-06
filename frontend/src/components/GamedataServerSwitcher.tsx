import { useCallback } from "react";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { GAMEDATA_SERVER_COOKIE, useI18n, useT } from "#/lib/i18n";
import { writePreferenceCookie } from "#/lib/i18n/cookie";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./GamedataServerSwitcher.messages";

/**
 * The value the pickers use for "no pick, follow the language". Not a server
 * code, so it can never collide with one.
 */
export const FOLLOW_LANGUAGE = "follow";

/**
 * A server's name in the reader's language. An unknown code (a server added
 * to the backend before its name was added here) shows as its code.
 */
export function useGamedataServerName(): (server: string) => string {
    const t: TypedT<typeof messages> = useT("common");
    return useCallback(
        (server: string) => {
            // Literal keys, one per server, so the extractor can see every name.
            switch (server) {
                case "en":
                    return t("gamedataServer.name.en");
                case "jp":
                    return t("gamedataServer.name.jp");
                case "kr":
                    return t("gamedataServer.name.kr");
                case "cn":
                    return t("gamedataServer.name.cn");
                case "tw":
                    return t("gamedataServer.name.tw");
                default:
                    return server.toUpperCase();
            }
        },
        [t],
    );
}

/**
 * The one place that knows how to change the game-data server: remember the
 * pick (or forget it, for {@link FOLLOW_LANGUAGE}), then reload.
 *
 * A reload rather than a cache swap because the server is resolved once per
 * request in the root bootstrap and threaded into every loader's query key.
 * Reloading re-runs that resolution server-side, so the first paint after the
 * switch already carries the new client's text, the same reasoning
 * `useLocaleSwitch` gives for the language.
 */
export function useGamedataServerSwitch(): (choice: string) => void {
    return useCallback((choice: string) => {
        // Awaited: a reload that races the write renders the old server.
        void writePreferenceCookie(GAMEDATA_SERVER_COOKIE, choice === FOLLOW_LANGUAGE ? null : choice).then(() => window.location.reload());
    }, []);
}

/** The picker's current value: the pick, or {@link FOLLOW_LANGUAGE}. */
export function useGamedataServerChoice(): string {
    const { gamedataServer, gamedataServerPicked } = useI18n();
    return gamedataServerPicked ? gamedataServer : FOLLOW_LANGUAGE;
}

/**
 * The picker's entries: "match language" first, then each loaded server.
 * Empty below two loaded servers, where there is nothing to choose between.
 */
export function useGamedataServerOptions(): { value: string; label: string }[] {
    const { gamedataServers, localeGamedataServer } = useI18n();
    const t: TypedT<typeof messages> = useT("common");
    const name = useGamedataServerName();
    if (gamedataServers.length < 2) return [];
    return [{ value: FOLLOW_LANGUAGE, label: t("gamedataServer.followLanguage", { server: name(localeGamedataServer) }) }, ...gamedataServers.map((server) => ({ value: server, label: name(server) }))];
}

/** The settings-page picker. Renders nothing below two loaded servers. */
export function GamedataServerSelect({ className }: { className?: string }): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("common");
    const options = useGamedataServerOptions();
    const choice = useGamedataServerChoice();
    const switchServer = useGamedataServerSwitch();

    if (options.length === 0) return null;
    const current = options.find((o) => o.value === choice)?.label ?? choice;

    return (
        <Select
            value={choice}
            onValueChange={(next: string | null) => {
                if (next !== null && next !== choice) switchServer(next);
            }}
        >
            <SelectTrigger className={className} aria-label={t("gamedataServer.choose")}>
                <SelectValue>{() => current}</SelectValue>
            </SelectTrigger>
            <SelectContent>
                {options.map((o) => (
                    <SelectItem key={o.value} value={o.value}>
                        {o.label}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}
