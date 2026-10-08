import { ReleasePlanner } from "frontend";

// The /tools/release page: header, the five tabs (Planner, Pulls, Events, Banners,
// Calendar) and the Planner tab underneath. Its data loads through server
// functions that reject in the design bundle, so the tab renders its real
// load-failure branch.
export const LoadFailed = () => <ReleasePlanner />;
