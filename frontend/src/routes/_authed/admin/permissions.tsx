import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Permissions } from "#/components/admin/screens/TierLists";

export const Route = createFileRoute("/_authed/admin/permissions")({
    component: AdminPermissionsRoute,
});

function AdminPermissionsRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Tier Lists" }]}>
            <Permissions />
        </AdminShell>
    );
}
