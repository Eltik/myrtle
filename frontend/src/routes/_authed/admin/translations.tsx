import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Translations } from "#/components/admin/screens/Translations";

export const Route = createFileRoute("/_authed/admin/translations")({
    component: AdminTranslationsRoute,
});

function AdminTranslationsRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Translations" }]}>
            <Translations />
        </AdminShell>
    );
}
