import { createFileRoute } from "@tanstack/react-router";
import { AdminShell } from "#/components/admin/AdminShell";
import { OperatorNotes } from "#/components/admin/screens/OperatorNotes";

export const Route = createFileRoute("/_authed/admin/operator-notes")({
    component: AdminNotesRoute,
});

function AdminNotesRoute(): React.ReactElement {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", to: "/" }, { label: "admin", to: "/admin" }, { label: "Operator notes" }]}>
            <OperatorNotes />
        </AdminShell>
    );
}
