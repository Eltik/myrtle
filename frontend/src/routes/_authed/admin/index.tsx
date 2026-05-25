import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Dashboard } from "#/components/admin/screens/Dashboard";

export const Route = createFileRoute("/_authed/admin/")({
    component: AdminDashboardRoute,
});

function AdminDashboardRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Dashboard" }]}>
            <Dashboard />
        </AdminShell>
    );
}
