import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/admin")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/login", search: { redirect: location.href } });
    },
    component: AdminLayout,
    head: () => {
        const { meta, links } = seo({
            title: "myrtle.moe · admin",
            description: "Operations console for myrtle.moe.",
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, { name: "robots", content: "noindex,nofollow" }, ...meta],
            links,
        };
    },
});

function AdminLayout(): React.ReactElement {
    return <Outlet />;
}
