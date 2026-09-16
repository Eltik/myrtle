import { createFileRoute, redirect } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Dashboard } from "#/components/admin/screens/Dashboard";
import { isAnyAdminRole } from "#/lib/api/admin";

export const Route = createFileRoute("/_authed/admin/")({
    // The parent gate admits translators, who reach the panel on a locale grant
    // rather than a staff role - and the dashboard is the one screen they are
    // guaranteed to be refused. Land them on the screen their grant is for
    // instead of on a 403.
    beforeLoad: ({ context }) => {
        if (!isAnyAdminRole(context.user?.role)) throw redirect({ to: "/admin/translations" });
    },
    component: AdminDashboardRoute,
});

function AdminDashboardRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Dashboard" }]}>
            <Dashboard />
        </AdminShell>
    );
}
