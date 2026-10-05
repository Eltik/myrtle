import { createFileRoute, notFound } from "@tanstack/react-router";
import { GridView } from "#/components/grids/GridView";
import { env } from "#/env";
import { gridQueryOptions } from "#/lib/api/grids";
import { metaT } from "#/lib/meta";
import { buildGridImageData, gridOgId } from "#/lib/og/impl/grid";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/grids_/$slug")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const grid = await context.queryClient.ensureQueryData(gridQueryOptions(params.slug, context.user?.id ?? null, context.i18n.gamedataServer));
        if (!grid) throw notFound();
        const server = context.i18n.gamedataServer;
        warmOg("grid-image", gridOgId(grid.slug, server), buildGridImageData(grid, env.VITE_BACKEND_URL ?? "", server));
        return grid;
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const locale = match.context.i18n?.locale;
        if (!loaderData) return seo({ title: t("grid.fallbackTitle"), path: `/grids/${params.slug}`, locale });
        return seo({
            title: loaderData.title,
            description: loaderData.description || t("grid.descriptionFallback", { rows: loaderData.rows, cols: loaderData.cols, owner: loaderData.owner.name }),
            path: `/grids/${loaderData.slug}`,
            image: ogURL("grid-image", gridOgId(loaderData.slug, match.context.i18n?.gamedataServer), buildGridImageData(loaderData, env.VITE_BACKEND_URL ?? "", match.context.i18n?.gamedataServer)),
            type: "article",
            locale,
        });
    },
});

function RouteComponent() {
    const { slug } = Route.useParams();
    return <GridView slug={slug} />;
}
