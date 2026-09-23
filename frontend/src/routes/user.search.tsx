import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { DEFAULT_SORT, parseAll, parseDir, parseOperatorId, parseOperatorIds, parseSort } from "#/components/user/search/impl/searchControls";
import { UserSearch } from "#/components/user/search/UserSearch";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

/**
 * `dir` defaults to absent: that means the sort's own default direction
 * (`defaultDir` in searchControls.ts), and the component only writes it when
 * the visitor flips away from that.
 */
const SEARCH_DEFAULTS = { q: "", page: 1, sort: DEFAULT_SORT, dir: undefined, has: "", support: "", all: "" } as const;

export const Route = createFileRoute("/user/search")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>) => {
        const rawPage = typeof search.page === "number" ? search.page : typeof search.page === "string" ? Number(search.page) : 1;
        const page = Number.isFinite(rawPage) && rawPage >= 1 ? Math.floor(rawPage) : 1;
        return {
            q: typeof search.q === "string" ? search.q : "",
            page,
            sort: parseSort(search.sort),
            dir: parseDir(search.dir),
            has: parseOperatorIds(search.has),
            support: parseOperatorId(search.support),
            all: parseAll(search.all),
        };
    },
    search: { middlewares: [stripSearchParams(SEARCH_DEFAULTS)] },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("userSearch.title"),
            description: t("userSearch.description"),
            path: "/user/search",
            image: defaultOgURL("user-search", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <UserSearch />;
}
