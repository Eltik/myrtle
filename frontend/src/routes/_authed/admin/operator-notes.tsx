import { createFileRoute, redirect } from "@tanstack/react-router";
import OperatorNotes from "#/components/admin/sections/OperatorNotes/OperatorNotes";
import { roleMayReachSection } from "#/components/admin/shell/model";
import { parseNotesSearch } from "#/components/admin/shell/search";

export const Route = createFileRoute("/_authed/admin/operator-notes")({
    validateSearch: parseNotesSearch,
    beforeLoad: ({ context }) => {
        if (!roleMayReachSection("notes", context.user?.role)) throw redirect({ to: "/admin" });
    },
    component: AdminOperatorNotesRoute,
});

function AdminOperatorNotesRoute(): React.ReactElement {
    return <OperatorNotes search={Route.useSearch()} />;
}
