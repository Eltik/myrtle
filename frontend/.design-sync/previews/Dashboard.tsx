import { Dashboard } from "frontend";

// `Dashboard` reads GET /admin/stats and /health through TanStack Query. In a
// preview both server functions are stubbed and there is no admin session, so
// `adminStatsQueryOptions(false)` stays disabled (skeleton tiles) while the
// health probe resolves to its unavailable branch. That pair is the screen's
// honest static state.
export function AwaitingAdminSession() {
    return <Dashboard />;
}
