import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { EnemiesList } from "#/components/enemies/list/Enemies";
import { enemiesQueryOptions, enemyStagesQueryOptions } from "#/lib/api/enemies";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

const SEARCH_DEFAULTS = { page: 1 } as const;

export const Route = createFileRoute("/enemies")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
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
    loader: async ({ context }) => {
        await Promise.all([
            context.queryClient.ensureQueryData(enemiesQueryOptions()),
            // Powers the "Appears In" location filter.
            context.queryClient.ensureQueryData(enemyStagesQueryOptions()),
        ]);
    },
    head: () => {
        const { meta, links } = seo({
            title: "Enemies",
            description: "View every enemy catalogued in Arknights.",
            path: "/enemies",
            image: defaultOgURL("enemies"),
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
    return <EnemiesList />;
}
