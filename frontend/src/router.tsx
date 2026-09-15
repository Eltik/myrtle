import { createRouter as createTanStackRouter } from "@tanstack/react-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query";
import { NotFound } from "#/components/NotFound";
import { DEFAULT_LOCALE, parseLocaleFromPath } from "#/lib/i18n/locale";
import { getContext } from "./integrations/tanstack-query/root-provider";
import { routeTree } from "./routeTree.gen";

/**
 * Locale lives in the router's location `rewrite`, not in the route tree.
 *
 * Two other approaches were tried and rejected, both for concrete reasons:
 *
 * 1. An optional `{-$locale}` segment wrapping every route. The installed
 *    router and file-based generator both support it - the generator emits a
 *    working tree for a `{-$locale}` directory over all 46 route files, which
 *    I verified. The cost is that every route's `fullPath` becomes
 *    `/{-$locale}/operators`, so all 99 `<Link to="/operators">` call sites
 *    plus 6 `navigate({to})` and 7 `redirect({to})` have to be rewritten and
 *    stay that way forever.
 *
 * 2. The `basepath` option, computed from the request URL. That type-checked,
 *    built clean, and did not work: `getRequestUrl()` is not available at the
 *    point `getRouter()` runs, so the basepath silently resolved to `/` on
 *    every request and `/ja` 404ed while the page around it still rendered in
 *    Japanese. Worth recording, because neither `tsc` nor the build caught it
 *    - only a rendered request did.
 *
 * `rewrite` needs no ambient request state at all: it is a pure function of
 * the URL, run on every location going in and out. `input` strips a locale
 * prefix before the router matches, so the route tree is untouched and
 * `to="/operators"` keeps working verbatim; `output` puts the prefix back when
 * an href is built, so links render as `/ja/operators`.
 *
 * English stays at the bare path, so every URL already indexed is unchanged
 * and nothing needs redirecting.
 */
function createAppRouter() {
    const context = getContext();

    // The locale for THIS router instance, learned from the first location it
    // parses. Per-instance rather than module-scope: `getRouter()` runs once
    // per request on the server, so a module-level value would leak one
    // visitor's language into another's concurrent render.
    const active = { locale: null as string | null };

    const router = createTanStackRouter({
        routeTree,
        context,
        rewrite: {
            input: ({ url }) => {
                const { locale, rest } = parseLocaleFromPath(url.pathname);
                if (locale) {
                    active.locale = locale;
                    url.pathname = rest;
                }
                return url;
            },
            output: ({ url }) => {
                if (active.locale && active.locale !== DEFAULT_LOCALE) {
                    url.pathname = `/${active.locale}${url.pathname === "/" ? "" : url.pathname}`;
                }
                return url;
            },
        },
        scrollRestoration: true,
        defaultPreload: "intent",
        defaultPreloadStaleTime: 0,
        defaultNotFoundComponent: NotFound,
    });

    setupRouterSsrQueryIntegration({ router, queryClient: context.queryClient });
    if (typeof window === "undefined") {
        const dehydrate = router.options.dehydrate;
        router.options.dehydrate = async () => {
            const dehydrated = await dehydrate?.();
            router.serverSsr?.onRenderFinished(() => {
                context.queryClient.clear();
            });
            return dehydrated;
        };
    }

    return router;
}

export function getRouter() {
    return createAppRouter();
}

declare module "@tanstack/react-router" {
    interface Register {
        router: ReturnType<typeof createAppRouter>;
    }
}
