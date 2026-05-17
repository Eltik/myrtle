import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { Users } from "#/components/admin/screens/Users";

export const Route = createFileRoute("/_authed/admin/users")({
    component: AdminUsersRoute,
});

function AdminUsersRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Users" }]}>
            <Users />
        </AdminShell>
    );
}
