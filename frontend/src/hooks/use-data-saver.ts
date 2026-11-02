import { useRouteContext } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { themeStore } from "#/lib/theme/store";

/**
 * Whether the reader turned on data saver (Settings, Appearance), which leaves
 * optional heavy art out until they ask for it.
 *
 * The setting lives in localStorage, which the server cannot read, so it is
 * mirrored into a cookie the root context carries. Until the theme store
 * hydrates, the context answers, so the server render and the first client
 * render agree and neither puts the full art in an `<img>`.
 */
export function useDataSaver(): boolean {
    const fromContext = useRouteContext({ from: "__root__", select: (c) => c.i18n?.dataSaver === true });
    const hydrated = useSelector(themeStore, (s) => s.hydrated);
    const stored = useSelector(themeStore, (s) => s.dataSaver);
    return hydrated ? stored : fromContext;
}
