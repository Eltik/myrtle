import { createFileRoute } from "@tanstack/react-router";
import { SettingsPage } from "#/components/settings/SettingsPage";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

// Not auth-gated: the Appearance section (theme, accent, dynamic art) is a
// client-side preference available to everyone. Account sections inside
// SettingsPage render only when signed in.
export const Route = createFileRoute("/settings")({
    component: RouteComponent,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("settings.title"),
            description: t("settings.description"),
            path: "/settings",
            image: defaultOgURL("settings", match.context.i18n),
            locale: match.context.i18n?.locale,
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
