import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { AdminSettings } from "#/components/admin/screens/Settings";

export const Route = createFileRoute("/_authed/admin/settings")({
    component: AdminSettingsRoute,
});

function AdminSettingsRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={["myrtle.moe", "admin", "Settings"]}>
            <AdminSettings />
        </AdminShell>
    );
}
