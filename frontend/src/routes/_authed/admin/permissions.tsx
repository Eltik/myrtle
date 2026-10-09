import { createFileRoute, redirect } from "@tanstack/react-router";

// Legacy URL: tier-list grants are the Tier lists section's Access tab.
export const Route = createFileRoute("/_authed/admin/permissions")({
    beforeLoad: () => {
        throw redirect({ to: "/admin/tier-lists", search: { tab: "access" }, replace: true });
    },
});
