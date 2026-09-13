import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { OperatorsList } from "#/components/operators/list/Operators";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorOwnershipQueryOptions, operatorsIndexQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

const SEARCH_DEFAULTS = { page: 1 } as const;

export const Route = createFileRoute("/operators")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // The page number lives in the URL so returning from an operator page lands
    // back on the page you left, with the scroll position the router restores.
    // Left optional so `<Link to="/operators">` still means "page one" and the
    // param only ever appears once you have paged somewhere else.
    validateSearch: (search: Record<string, unknown>): { page?: number } => {
        const raw = typeof search.page === "number" ? search.page : typeof search.page === "string" ? Number(search.page) : 1;
        const page = Number.isFinite(raw) && raw >= 1 ? Math.floor(raw) : 1;
        return page === 1 ? {} : { page };
    },
    // `validateSearch` only runs on the way in, so page one still needs stripping
    // on the way out to keep a bare `/operators` in the address bar.
    search: { middlewares: [stripSearchParams(SEARCH_DEFAULTS)] },
    loader: ({ context }) => Promise.all([context.queryClient.ensureQueryData(operatorsIndexQueryOptions()), context.queryClient.prefetchQuery(upcomingQueryOptions()), context.queryClient.prefetchQuery(operatorNotesListQueryOptions()), context.queryClient.prefetchQuery(operatorOwnershipQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "Operators",
            description: "View all operators released in Arknights.",
            path: "/operators",
            image: defaultOgURL("operators"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RootErrorComponent({ error }: { error: unknown }) {
    console.error("Router error:", error);

    return (
        <div style={{ padding: 20 }}>
            <h1>Something went wrong</h1>
            <pre style={{ color: "red" }}>{error instanceof Error ? error.message : JSON.stringify(error, null, 2)}</pre>
            <pre>{error instanceof Error ? error.stack : null}</pre>
        </div>
    );
}

function RouteComponent() {
    return <OperatorsList />;
}
