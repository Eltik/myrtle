import { createFileRoute, redirect } from "@tanstack/react-router";
import TierLists from "#/components/admin/sections/TierLists/TierLists";
import { roleMayReachSection } from "#/components/admin/shell/model";
import { parseTierListsSearch } from "#/components/admin/shell/search";

export const Route = createFileRoute("/_authed/admin/tier-lists")({
    validateSearch: parseTierListsSearch,
    beforeLoad: ({ context }) => {
        if (!roleMayReachSection("tierlists", context.user?.role)) throw redirect({ to: "/admin" });
    },
    component: AdminTierListsRoute,
});

function AdminTierListsRoute(): React.ReactElement {
    return <TierLists search={Route.useSearch()} />;
}
