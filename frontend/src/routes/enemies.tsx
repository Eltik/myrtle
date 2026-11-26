import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { EnemiesList } from "#/components/enemies/list/Enemies";
import { RouteError } from "#/components/ui/route-error";
import { enemiesQueryOptions } from "#/lib/api/enemies";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

const SEARCH_DEFAULTS = { page: 1 } as const;

export const Route = createFileRoute("/enemies")({
    component: RouteComponent,
    errorComponent: RouteError,
    // The page number lives in the URL so returning from an enemy page lands
    // back on the page you left, with the scroll position the router restores.
    // Left optional so `<Link to="/enemies">` still means "page one" and the
    // param only ever appears once you have paged somewhere else.
    validateSearch: (search: Record<string, unknown>): { page?: number } => {
        const raw = typeof search.page === "number" ? search.page : typeof search.page === "string" ? Number(search.page) : 1;
        const page = Number.isFinite(raw) && raw >= 1 ? Math.floor(raw) : 1;
        return page === 1 ? {} : { page };
    },
    // `validateSearch` only runs on the way in, so page one still needs stripping
    // on the way out to keep a bare `/enemies` in the address bar.
    search: { middlewares: [stripSearchParams(SEARCH_DEFAULTS)] },
    // The active locale's game-data server, so a locale pinned to `jp` warms
    // the Japanese enemy table rather than the English one. The stage table
    // behind the "Appears In" filter is left to the client: dehydrated, it
    // was 3,662,632 B of the 6,175,372 B page.
    loader: async ({ context }) => {
        await context.queryClient.ensureQueryData(enemiesQueryOptions(context.i18n.gamedataServer));
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("enemies.title"),
            description: t("enemies.description"),
            path: "/enemies",
            image: defaultOgURL("enemies", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <EnemiesList />;
}
