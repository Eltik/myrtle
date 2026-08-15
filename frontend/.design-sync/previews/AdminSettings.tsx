import { AdminSettings } from "frontend";

// Both panels are query-backed (GET /health, GET /admin/stats) and there is no
// signed-in session in a preview, so the screen renders its probe-failed branch
// with the session card falling back to placeholders.
export function ProbeUnavailable() {
    return <AdminSettings />;
}
