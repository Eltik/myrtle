import { createFileRoute, redirect } from "@tanstack/react-router";

// Legacy URL: the audit log is the System section's Edit history tab.
export const Route = createFileRoute("/_authed/admin/audit")({
    beforeLoad: () => {
        throw redirect({ to: "/admin/system", search: { tab: "audit" }, replace: true });
    },
});
