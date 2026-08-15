import { HpsCalculator } from "frontend";

// The whole healing tool page. Same shape as the DPS calculator: reducer state
// plus a server-function operator list that is stubbed in previews, so the card
// shows the first-run page with the default team buffs and an empty chart.
export const HealerListUnavailable = () => <HpsCalculator />;
