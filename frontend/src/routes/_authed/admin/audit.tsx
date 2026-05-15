import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Audit } from "#/components/admin/screens/Audit";

export const Route = createFileRoute("/_authed/admin/audit")({
    component: AdminAuditRoute,
});

function AdminAuditRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={["myrtle.moe", "admin", "Audit log"]}>
            <Audit />
        </AdminShell>
    );
}
