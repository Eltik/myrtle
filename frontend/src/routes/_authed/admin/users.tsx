import { createFileRoute, redirect } from "@tanstack/react-router";
import Users from "#/components/admin/sections/Users/Users";
import { roleMayReachSection } from "#/components/admin/shell/model";
import { parseUsersSearch } from "#/components/admin/shell/search";

export const Route = createFileRoute("/_authed/admin/users")({
    validateSearch: parseUsersSearch,
    beforeLoad: ({ context }) => {
        if (!roleMayReachSection("users", context.user?.role)) throw redirect({ to: "/admin" });
    },
    component: AdminUsersRoute,
});

function AdminUsersRoute(): React.ReactElement {
    return <Users search={Route.useSearch()} />;
}
