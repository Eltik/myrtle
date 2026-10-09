import { createFileRoute, redirect } from "@tanstack/react-router";
import System from "#/components/admin/sections/System/System";
import { roleMayReachSection } from "#/components/admin/shell/model";
import { parseSystemSearch } from "#/components/admin/shell/search";

export const Route = createFileRoute("/_authed/admin/system")({
    validateSearch: parseSystemSearch,
    beforeLoad: ({ context }) => {
        if (!roleMayReachSection("system", context.user?.role)) throw redirect({ to: "/admin" });
    },
    component: AdminSystemRoute,
});

function AdminSystemRoute(): React.ReactElement {
    return <System search={Route.useSearch()} />;
}
