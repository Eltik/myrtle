import { createFileRoute, redirect } from "@tanstack/react-router";
import { Suspense } from "react";
import { TierListEditor } from "#/components/tier-lists/edit/TierListEditor";
import { Spinner } from "#/components/ui/spinner";
import { DEFAULT_ENTITY_KINDS } from "#/lib/api/tier-entities";
import { tierEntityCatalogueQueryOptions, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { metaT } from "#/lib/meta";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/tier-lists_/my_/$id/edit")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/", search: { auth: "1", next: location.href } });
    },
    loader: ({ context, params }) => {
        const server = context.i18n.gamedataServer;
        // Warm the default kinds at once, then whatever else the list offers once its detail arrives. Not awaited: the editor suspends on the detail itself.
        for (const kind of DEFAULT_ENTITY_KINDS) void context.queryClient.prefetchQuery(tierEntityCatalogueQueryOptions(kind, server));
        void context.queryClient
            .ensureQueryData(tierListDetailQueryOptions(params.id, server))
            .then((detail) => {
                for (const kind of detail?.entityKinds ?? []) void context.queryClient.prefetchQuery(tierEntityCatalogueQueryOptions(kind, server));
            })
            .catch(() => undefined);
    },
    component: RouteComponent,
    head: ({ match, params }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("editTierList.title"),
            description: t("editTierList.description"),
            path: `/tier-lists/my/${params.id}/edit`,
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, { name: "robots", content: "noindex,nofollow" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    const { id } = Route.useParams();
    return (
        <Suspense fallback={<EditorLoading />}>
            <TierListEditor slug={id} />
        </Suspense>
    );
}

function EditorLoading() {
    return (
        <main className="grid min-h-dvh place-items-center">
            <div className="flex items-center gap-2 font-sans text-muted-foreground text-sm">
                <Spinner />
                <span>Loading editor…</span>
            </div>
        </main>
    );
}
