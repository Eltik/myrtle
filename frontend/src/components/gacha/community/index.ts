// v3: Community gacha stats simplified - enhanced stats no longer available.
// Components in this directory are deprecated. The community page now renders inline.
export type { ActualRates } from "./impl/helpers";
export { calculateActualRates, calculateLuckScore, buildRateComparisonData, calculateDerivedData, buildRarityData, groupOperatorsByRarity, transformHourlyData, transformDailyData, transformDateData } from "./impl/helpers";
export type { LuckStatus } from "./impl/constants";
export { EXPECTED_RATES, RARITY_COLORS, getLuckStatus } from "./impl/constants";
