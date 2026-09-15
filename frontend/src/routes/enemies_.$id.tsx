import { createFileRoute } from "@tanstack/react-router";
import { EnemyDetail } from "#/components/enemies/detail/Enemies";
import { enemyDetailQueryOptions } from "#/lib/api/enemies";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/enemies_/$id")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // Only the single enemy record + its race lookup are needed for the default
    // (Overview) tab. The "Appears In" and Chibi tabs fetch their own data lazily
    // when opened, so nothing else is warmed here.
    loader: ({ context, params }) => context.queryClient.ensureQueryData(enemyDetailQueryOptions(params.id, context.i18n.gamedataServer)),
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const locale = match.context.i18n?.locale;
        const enemy = loaderData?.enemy ?? null;
        if (!enemy) return seo({ title: t("enemy.fallbackTitle"), path: `/enemies/${params.id}`, locale });
        // The enemy's own code and blurb come from game data, which ships its
        // own per-region text; only the no-blurb fallback is a message.
        const desc = [enemy.enemyIndex, enemy.description].filter(Boolean).join(" - ") || t("enemy.descriptionFallback", { index: enemy.enemyIndex });
        const { meta, links } = seo({
            title: enemy.name,
            description: desc,
            path: `/enemies/${params.id}`,
            image: defaultOgURL("enemies", match.context.i18n),
            type: "article",
            locale,
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
