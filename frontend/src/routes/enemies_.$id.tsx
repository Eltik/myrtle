import { createFileRoute } from "@tanstack/react-router";
import { EnemyDetail } from "#/components/enemies/detail/Enemies";
import { enemyDetailQueryOptions } from "#/lib/api/enemies";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/enemies_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // Only the single enemy record + its race lookup are needed for the default
    // (Overview) tab. The "Appears In" and Chibi tabs fetch their own data lazily
    // when opened, so nothing else is warmed here.
    loader: ({ context, params }) => context.queryClient.ensureQueryData(enemyDetailQueryOptions(params.id)),
    head: ({ loaderData, params }) => {
        const enemy = loaderData?.enemy ?? null;
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
