import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./use-player-name.messages";

/** The name to show for a signed-in visitor: their in-game nickname, or the localized fallback when the game sent none. */
export function usePlayerName(): (nickname: string | null | undefined) => string {
    const t: TypedT<typeof messages> = useT("common");
    return (nickname) => nickname ?? t("playerFallback");
}
