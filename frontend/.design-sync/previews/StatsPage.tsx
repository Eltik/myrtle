import { StatsPage } from "frontend";

/**
 * StatsPage owns its own `GET /stats` query. Server functions are stubbed in a
 * preview, so the honest render is the error branch: the page header and the
 * catalog/tier-list scaffolding stay up while every figure falls back to `-`.
 */
export const StatsUnavailable = () => <StatsPage />;
