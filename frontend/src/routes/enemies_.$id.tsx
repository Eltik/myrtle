import { createFileRoute } from "@tanstack/react-router";
import { EnemyDetail } from "#/components/enemies/detail/Enemies";
import { enemiesQueryOptions, enemyStagesQueryOptions } from "#/lib/api/enemies";
import { zonesQueryOptions } from "#/lib/api/stages";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/enemies_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        const [handbook] = await Promise.all([
            context.queryClient.ensureQueryData(enemiesQueryOptions()),
            // Stage appearances + zones power the "Appears In" tab; warm them so
            // the tab renders without a client fetch flash.
            context.queryClient.ensureQueryData(enemyStagesQueryOptions()),
            context.queryClient.ensureQueryData(zonesQueryOptions()),
        ]);
        return handbook.enemyData[params.id] ?? null;
    },
    head: ({ loaderData, params }) => {
        const enemy = loaderData;
        if (!enemy) return seo({ title: "Enemy", path: `/enemies/${params.id}` });
        const desc = [enemy.enemyIndex, enemy.description].filter(Boolean).join(" - ") || `Enemy ${enemy.enemyIndex}`;
        const { meta, links } = seo({
            title: enemy.name,
            description: desc,
            path: `/enemies/${params.id}`,
            image: defaultOgURL("enemies"),
            type: "article",
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
    return <EnemyDetail />;
}
