import type { GamedataServer } from "#/lib/api/gamedata";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

/** What each game region means to a reader picking one. Shared with the sheet. */
export function useServerLabels(): Record<GamedataServer, string> {
    const t: SystemT = useT("admin");
    return {
        en: t("system.languages.server.en"),
        jp: t("system.languages.server.jp"),
        kr: t("system.languages.server.kr"),
        cn: t("system.languages.server.cn"),
        tw: t("system.languages.server.tw"),
        bili: t("system.languages.server.bili"),
    };
}
