import { createFileRoute, redirect } from "@tanstack/react-router";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/settings")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/login", search: { redirect: location.href } });
    },
    component: RouteComponent,
    head: () => {
        const { meta, links } = seo({
            title: "Settings",
            description: "Manage your account, profile, and preferences.",
            path: "/settings",
            image: defaultOgURL("settings"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <div>Hello "/_authed/settings"!</div>;
}
