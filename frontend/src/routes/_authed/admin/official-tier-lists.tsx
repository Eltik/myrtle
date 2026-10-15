import { createFileRoute, redirect } from "@tanstack/react-router";

// Legacy URL: official lists are the Tier lists section's Lists tab.
export const Route = createFileRoute("/_authed/admin/official-tier-lists")({
    beforeLoad: () => {
        throw redirect({ to: "/admin/tier-lists", replace: true });
    },
});
