import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { OfficialTierLists } from "#/components/admin/screens/OfficialTierLists";

export const Route = createFileRoute("/_authed/admin/official-tier-lists")({
    component: AdminOfficialRoute,
});

function AdminOfficialRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Official tier lists" }]}>
            <OfficialTierLists />
        </AdminShell>
    );
}
