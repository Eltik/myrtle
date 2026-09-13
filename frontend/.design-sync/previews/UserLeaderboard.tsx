import { LeaderboardLeaderboard } from "frontend";

// The full leaderboard page container: breadcrumb, hero, toolbar, table, pagination
// and the right-hand standing/movers rail. It reads typed search params from the
// `/user/leaderboard` route and fetches its own pages, so in the design bundle it
// renders the empty state its stubbed queries settle into.
export const PageShell = () => <LeaderboardLeaderboard />;
