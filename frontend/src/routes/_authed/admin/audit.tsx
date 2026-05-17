import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Audit } from "#/components/admin/screens/Audit";

export const Route = createFileRoute("/_authed/admin/audit")({
    component: AdminAuditRoute,
});

function AdminAuditRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Audit log" }]}>
            <Audit />
        </AdminShell>
    );
}
