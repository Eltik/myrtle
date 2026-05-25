import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Health } from "#/components/admin/screens/Health";

export const Route = createFileRoute("/_authed/admin/health")({
    component: AdminHealthRoute,
});

function AdminHealthRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Health & cache" }]}>
            <Health />
        </AdminShell>
    );
}
