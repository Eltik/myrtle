import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Permissions } from "#/components/admin/screens/Permissions";

export const Route = createFileRoute("/_authed/admin/permissions")({
    component: AdminPermissionsRoute,
});

function AdminPermissionsRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={["myrtle.moe", "admin", "Tier Lists"]}>
            <Permissions />
        </AdminShell>
    );
}
