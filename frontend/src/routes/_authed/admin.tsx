import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { metaT } from "#/lib/meta";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/_authed/admin")({
    beforeLoad: ({ context, location }) => {
        if (!context.user) throw redirect({ to: "/", search: { auth: "1", next: location.href } });
        // The server's answer, not a role string reinterpreted here: it is
        // resolved from the database role AND from `translation_permissions`,
        // so a translator holding only a locale grant gets in, and this gate
        // cannot disagree with the ones the admin routes apply.
        if (!context.user.canAccessAdminPanel) throw redirect({ to: "/" });
    },
    component: AdminLayout,
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("admin.title"),
            description: t("admin.description"),
            locale: match.context.i18n?.locale,
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
