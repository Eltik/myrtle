import { createFileRoute } from "@tanstack/react-router";
import { SettingsPage } from "#/components/settings/SettingsPage";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

// Not auth-gated: the Appearance section (theme, accent, dynamic art) is a
// client-side preference available to everyone. Account sections inside
// SettingsPage render only when signed in.
export const Route = createFileRoute("/settings")({
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
    const { user } = Route.useRouteContext();
    return <SettingsPage user={user ?? null} />;
}
