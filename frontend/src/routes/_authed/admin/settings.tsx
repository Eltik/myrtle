import { createFileRoute, redirect } from "@tanstack/react-router";

// Legacy URL: site languages moved to the System section; game data is on its Health tab.
export const Route = createFileRoute("/_authed/admin/settings")({
    beforeLoad: () => {
        throw redirect({ to: "/admin/system", search: { tab: "languages" }, replace: true });
    },
});
