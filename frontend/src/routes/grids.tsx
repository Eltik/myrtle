import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { GridsBrowse } from "#/components/grids/GridsBrowse";
import { DEFAULT_GRIDS_SEARCH, type IGridsSearch } from "#/components/grids/shared";
import { browseGridsQueryOptions } from "#/lib/api/grids";
import { metaT } from "#/lib/meta";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/grids")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): IGridsSearch => {
        const sort = search.sort === "popular" ? "popular" : "recent";
        const q = typeof search.q === "string" ? search.q : "";
        const raw = typeof search.page === "number" ? search.page : Number.parseInt(String(search.page ?? ""), 10);
        const page = Number.isFinite(raw) && raw >= 1 ? Math.floor(raw) : 1;
        return { sort, q, page };
    },
    search: { middlewares: [stripSearchParams(DEFAULT_GRIDS_SEARCH)] },
    loaderDeps: ({ search }) => search,
    loader: ({ context, deps }) => context.queryClient.ensureQueryData(browseGridsQueryOptions({ ...deps, server: context.i18n.gamedataServer })),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({ title: t("grids.title"), description: t("grids.description"), path: "/grids", locale: match.context.i18n?.locale });
        return { meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta], links };
    },
});

function RouteComponent() {
    return <GridsBrowse search={Route.useSearch()} />;
}
