import { createFileRoute } from "@tanstack/react-router";
import { EnemiesList } from "#/components/enemies/list/Enemies";
import { enemiesQueryOptions, enemyStagesQueryOptions } from "#/lib/api/enemies";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/enemies")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
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
