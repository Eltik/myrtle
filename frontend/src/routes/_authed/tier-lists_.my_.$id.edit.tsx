import { createFileRoute, redirect } from "@tanstack/react-router";
import { Suspense } from "react";
import { TierListEditor } from "#/components/tier-lists/edit/TierListEditor";
import { Spinner } from "#/components/ui/spinner";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/tier-lists_/my_/$id/edit")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/login", search: { redirect: location.href } });
    },
    loader: ({ context, params }) => {
        void context.queryClient.prefetchQuery(tierListDetailQueryOptions(params.id));
        void context.queryClient.prefetchQuery(operatorsIndexQueryOptions());
    },
    component: RouteComponent,
    head: ({ params }) => {
        const { meta, links } = seo({
            title: "Edit Tier List",
            description: "Edit your tier list.",
            path: `/tier-lists/my/${params.id}/edit`,
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
