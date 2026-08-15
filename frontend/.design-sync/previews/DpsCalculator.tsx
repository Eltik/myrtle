import { DpsCalculator } from "frontend";

// The whole DPS tool page. It owns its state (reducer + localStorage) and pulls
// its operator list from a server function, which is stubbed in previews - so
// the honest card is the first-run page: empty picker, default enemy, and the
// chart's own "No operators yet" empty state.
export const OperatorListUnavailable = () => <DpsCalculator />;
