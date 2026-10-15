import { createFileRoute, redirect } from "@tanstack/react-router";

// Legacy URL: health moved into the System section.
export const Route = createFileRoute("/_authed/admin/health")({
    beforeLoad: () => {
        throw redirect({ to: "/admin/system", search: { tab: "health" }, replace: true });
    },
});
