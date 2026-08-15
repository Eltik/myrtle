import { Randomizer } from "frontend";

// The whole randomizer tool page. It reads operators, stages, zones, activities
// and (when signed in) the user's roster through server functions, which are
// stubbed in previews — so the honest card is the first-run page: the briefing
// header with its counters at zero, "Roll squad" disabled, and the pre-roll
// empty state below it.
export const NoStageDataAvailable = () => <Randomizer />;
