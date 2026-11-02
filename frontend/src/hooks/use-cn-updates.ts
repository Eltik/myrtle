import { useSelector } from "@tanstack/react-store";
import { themeStore } from "#/lib/theme/store";

/**
 * Whether CN's updates show for a reader on `server` (Settings, Appearance, on by default).
 *
 * False until the theme store hydrates, so a stored opt-out is read before any CN request starts,
 * and always false on CN itself, which has nothing newer to compare against.
 */
export function useCnUpdates(server: string): boolean {
    const hydrated = useSelector(themeStore, (s) => s.hydrated);
    const enabled = useSelector(themeStore, (s) => s.cnUpdates);
    return hydrated && enabled && server !== "cn";
}
