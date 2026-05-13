import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { Browse } from "#/components/tier-lists/Browse";
import { SORT_OPTIONS, type TierListSort, type TierListType } from "#/components/tier-lists/FilterToolbar";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

interface ITierListsSearch {
    type: TierListType;
    sort: TierListSort;
    q: string;
    flair: string[];
}

const DEFAULTS: ITierListsSearch = { type: "all", sort: "trending", q: "", flair: [] };

const VALID_TYPES = new Set<TierListType>(["all", "official", "community"]);
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
    loader: ({ context }) => context.queryClient.ensureQueryData(browseTierListsQueryOptions()),
    validateSearch: (search: Record<string, unknown>): ITierListsSearch => {
        const typeRaw = typeof search.type === "string" ? (search.type as TierListType) : "all";
        const type: TierListType = VALID_TYPES.has(typeRaw) ? typeRaw : "all";
        const sortRaw = typeof search.sort === "string" ? (search.sort as TierListSort) : "trending";
        const sort: TierListSort = VALID_SORTS.has(sortRaw) ? sortRaw : "trending";
        const q = typeof search.q === "string" ? search.q : "";
        const flair = parseFlair(search.flair);
        return { type, sort, q, flair };
    },
    search: { middlewares: [stripSearchParams(DEFAULTS)] },
    head: () => {
        const { meta, links } = seo({
            title: "Tier Lists",
            description: "Browse official and community tier lists for Arknights operators.",
            path: "/tier-lists",
            image: defaultOgURL("tier-lists"),
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
