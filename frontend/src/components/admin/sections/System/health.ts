import type { IHealthResponse } from "#/lib/api/admin";

/**
 * What the status line under the System head says. `checking` is the first
 * probe still in flight; `unreachable` is the probe itself failing, which is a
 * different story from the backend answering and reporting a sick service.
 */
export type HealthSummary = "checking" | "unreachable" | "healthy" | "databaseDown" | "cacheDown" | "bothDown" | "degraded";

export type HealthDot = "green" | "amber" | "red";

export function summarizeHealth(health: IHealthResponse | undefined, failed: boolean): HealthSummary {
    if (!health) return failed ? "unreachable" : "checking";
    const dbUp = health.database.status === "connected";
    const cacheUp = health.cache.status === "connected";
    if (!dbUp && !cacheUp) return "bothDown";
    if (!dbUp) return "databaseDown";
    if (!cacheUp) return "cacheDown";
    // Both probes answered but the backend still calls itself degraded.
    return health.status === "ok" ? "healthy" : "degraded";
}

/**
 * A cache outage only slows the site (every read falls through to the
 * database), so it is amber. Anything that loses data or the backend is red.
 */
export function healthDot(summary: HealthSummary): HealthDot {
    switch (summary) {
        case "healthy":
            return "green";
        case "checking":
        case "cacheDown":
        case "degraded":
            return "amber";
        case "unreachable":
        case "databaseDown":
        case "bothDown":
            return "red";
    }
}
