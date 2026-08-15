import { HistoryPage } from "frontend";

/**
 * The whole /gacha/history route. Its records come from an authenticated
 * server function, and a preview has no session, so it renders the signed-out
 * gate under the real page header.
 */
export const SignedOut = () => <HistoryPage />;
