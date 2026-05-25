import { createFileRoute, redirect, stripSearchParams } from "@tanstack/react-router";
import { useCallback } from "react";
import { MyTierLists } from "#/components/tier-lists/my/MyTierLists";
import { MY_SORT_OPTIONS, type MyListSort, type MyListTypeFilter, type MyViewMode } from "#/components/tier-lists/my/MyToolbar";
import { myTierListsDetailedQueryOptions } from "#/lib/api/tier-lists";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

interface IMyTierListsSearch {
    sort: MyListSort;
    type: MyListTypeFilter;
    view: MyViewMode;
    q: string;
}

const DEFAULTS: IMyTierListsSearch = { sort: "recent", type: "all", view: "grid", q: "" };

const VALID_SORTS = new Set<MyListSort>(MY_SORT_OPTIONS.map((s) => s.value));
const VALID_TYPES = new Set<MyListTypeFilter>(["all", "community", "official"]);
const VALID_VIEWS = new Set<MyViewMode>(["grid", "list"]);

export const Route = createFileRoute("/_authed/tier-lists_/my")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/", search: { auth: "1", next: location.href } });
    },
    component: RouteComponent,
    loader: ({ context }) => context.queryClient.ensureQueryData(myTierListsDetailedQueryOptions(Boolean(context.user))),
    validateSearch: (search: Record<string, unknown>): IMyTierListsSearch => {
        const sortRaw = typeof search.sort === "string" ? (search.sort as MyListSort) : DEFAULTS.sort;
        const sort: MyListSort = VALID_SORTS.has(sortRaw) ? sortRaw : DEFAULTS.sort;
        const typeRaw = typeof search.type === "string" ? (search.type as MyListTypeFilter) : DEFAULTS.type;
        const type: MyListTypeFilter = VALID_TYPES.has(typeRaw) ? typeRaw : DEFAULTS.type;
        const viewRaw = typeof search.view === "string" ? (search.view as MyViewMode) : DEFAULTS.view;
        const view: MyViewMode = VALID_VIEWS.has(viewRaw) ? viewRaw : DEFAULTS.view;
        const q = typeof search.q === "string" ? search.q : "";
        return { sort, type, view, q };
    },
    search: { middlewares: [stripSearchParams(DEFAULTS)] },
    head: () => {
        const { meta, links } = seo({
            title: "My Tier Lists",
            description: "Manage, edit, and share the tier lists you've created.",
            path: "/tier-lists/my",
            image: defaultOgURL("tier-lists"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, { name: "robots", content: "noindex,nofollow" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const search = Route.useSearch();
    const navigate = Route.useNavigate();

    const persistSearch = useCallback(
        (next: { sort: MyListSort; type: MyListTypeFilter; view: MyViewMode; q: string }) => {
            if (next.sort === search.sort && next.type === search.type && next.view === search.view && next.q === search.q) {
                return;
            }
            navigate({ search: () => next, replace: true, resetScroll: false });
        },
        [navigate, search.sort, search.type, search.view, search.q],
    );

    return <MyTierLists initialSort={search.sort} initialType={search.type} initialView={search.view} initialQuery={search.q} onPersistSearch={persistSearch} />;
}
