import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { UserSearch } from "#/components/user/search/UserSearch";
import { seo } from "#/lib/seo";

const SEARCH_DEFAULTS = { q: "", page: 1 } as const;

export const Route = createFileRoute("/user/search")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>) => {
        const rawPage = typeof search.page === "number" ? search.page : typeof search.page === "string" ? Number(search.page) : 1;
        const page = Number.isFinite(rawPage) && rawPage >= 1 ? Math.floor(rawPage) : 1;
        return {
            q: typeof search.q === "string" ? search.q : "",
            page,
        };
    },
    search: { middlewares: [stripSearchParams(SEARCH_DEFAULTS)] },
    head: () => {
        const { meta, links } = seo({
            title: "Search Doctors",
            description: "Find and browse user profiles.",
            path: "/user/search",
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
