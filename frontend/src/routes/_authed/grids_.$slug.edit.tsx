import { createFileRoute, redirect } from "@tanstack/react-router";
import { Suspense } from "react";
import { GridEditor } from "#/components/grids/GridEditor";
import { Spinner } from "#/components/ui/spinner";
import { gridQueryOptions } from "#/lib/api/grids";
import { tierEntityCatalogueQueryOptions } from "#/lib/api/tier-lists";
import { metaT } from "#/lib/meta";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/grids_/$slug/edit")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/", search: { auth: "1", next: location.href } });
    },
    loader: ({ context, params }) => {
        const server = context.i18n.gamedataServer;
        // Warm the operator catalogue, the picker's first tab on most grids. Not awaited: the editor suspends on the grid itself.
        void context.queryClient.prefetchQuery(tierEntityCatalogueQueryOptions("operator", server));
        void context.queryClient.prefetchQuery(gridQueryOptions(params.slug, context.user?.id ?? null, server));
    },
    component: RouteComponent,
    head: ({ match, params }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({ title: t("editGrid.title"), description: t("editGrid.description"), path: `/grids/${params.slug}/edit`, locale: match.context.i18n?.locale });
        return { meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, { name: "robots", content: "noindex,nofollow" }, ...meta], links };
    },
});

function RouteComponent() {
    const { slug } = Route.useParams();
    return (
        <Suspense fallback={<EditorLoading />}>
            <GridEditor slug={slug} />
        </Suspense>
    );
}

function EditorLoading() {
    return (
        <main className="grid min-h-dvh place-items-center">
            <Spinner />
        </main>
    );
}
