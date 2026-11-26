import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { Browse } from "#/components/tier-lists/Browse";
import { SORT_OPTIONS, type TierListSort, type TierListType } from "#/components/tier-lists/FilterToolbar.constants";
import { browseTierListsQueryOptions, isAccountId, tierListFlairsQueryOptions } from "#/lib/api/tier-lists";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

interface ITierListsSearch {
    type: TierListType;
    sort: TierListSort;
    q: string;
    flair: string[];
    /** An account id: only that player's listed lists. */
    author?: string;
}

const DEFAULTS: ITierListsSearch = { type: "all", sort: "recent", q: "", flair: [] };

const VALID_TYPES = new Set<TierListType>(["all", "official", "community", "favorites"]);
const VALID_SORTS = new Set<TierListSort>(SORT_OPTIONS.map((s) => s.value));

function parseFlair(input: unknown): string[] {
    if (Array.isArray(input)) {
        return input.filter((v): v is string => typeof v === "string" && v.length > 0);
    }
    if (typeof input === "string" && input.length > 0) {
        return input
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean);
    }
    return [];
}

export const Route = createFileRoute("/tier-lists")({
    component: RouteComponent,
    loaderDeps: ({ search }) => ({ author: search.author }),
    loader: async ({ context, deps }) => {
        const server = context.i18n.gamedataServer;
        await Promise.all([context.queryClient.ensureQueryData(browseTierListsQueryOptions(server)), deps.author ? context.queryClient.ensureQueryData(browseTierListsQueryOptions(server, deps.author)) : null, context.queryClient.ensureQueryData(tierListFlairsQueryOptions())]);
    },
    validateSearch: (search: Record<string, unknown>): ITierListsSearch => {
        const typeRaw = typeof search.type === "string" ? (search.type as TierListType) : "all";
        const type: TierListType = VALID_TYPES.has(typeRaw) ? typeRaw : "all";
        const sortRaw = typeof search.sort === "string" ? (search.sort as TierListSort) : "recent";
        const sort: TierListSort = VALID_SORTS.has(sortRaw) ? sortRaw : "recent";
        const q = typeof search.q === "string" ? search.q : "";
        const flair = parseFlair(search.flair);
        const author = typeof search.author === "string" && isAccountId(search.author) ? search.author.toLowerCase() : undefined;
        return { type, sort, q, flair, author };
    },
    search: { middlewares: [stripSearchParams(DEFAULTS)] },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("tierLists.title"),
            description: t("tierLists.description"),
            path: "/tier-lists",
            image: defaultOgURL("tier-lists", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <Browse />;
}
