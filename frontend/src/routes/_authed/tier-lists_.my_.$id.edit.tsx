import { createFileRoute, redirect } from "@tanstack/react-router";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/tier-lists_/my_/$id/edit")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/login", search: { redirect: location.href } });
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
    return <main className="min-h-dvh" />;
}
