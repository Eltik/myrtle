import { CommunityPage } from "frontend";

/**
 * The whole /gacha/community route. It owns four queries (enhanced stats,
 * operator index, banners, per-banner totals) and server functions are stubbed
 * in a preview, so the honest render is the error branch: the header and the
 * outcome-mix panel stay up while the figures fall back to `-`.
 */
export const CommunityStatsUnavailable = () => <CommunityPage />;
