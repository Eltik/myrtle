import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { AdminSettings } from "#/components/admin/screens/Settings";

export const Route = createFileRoute("/_authed/admin/settings")({
    component: AdminSettingsRoute,
});

function AdminSettingsRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Settings" }]}>
            <AdminSettings />
        </AdminShell>
    );
}
