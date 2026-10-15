import { LeaderboardToolbar } from "frontend";

// The leaderboard's filter row: rank-by picker, scope, server, interval,
// movement-only and the player search. The rank-by picker reads the item
// catalog; before it loads the page passes the empty catalog.

const noop = () => {};
const EMPTY_CATALOG = { items: [], population: null };
const CATALOG = {
    items: [{ item_id: "4002", holders: 2412, top: 6421, total_quantity: 1873390, meta: null, name: "Originite Prime", rarityNum: 6, iconId: "DIAMOND" }],
    population: 2629,
};
const base = { onRanking: noop, materials: undefined as never, onInterval: noop, onMovementOnly: noop, onQuery: noop, onScope: noop, onServer: noop };

export const Default = () => <LeaderboardToolbar {...base} ranking={{ kind: "score", sort: "total_score" }} catalog={EMPTY_CATALOG} interval="1 day" movementOnly={false} query="" scope="global" server="All" />;

export const MovementOnly = () => <LeaderboardToolbar {...base} ranking={{ kind: "score", sort: "operator_score" }} catalog={EMPTY_CATALOG} interval="7 days" movementOnly={true} query="" scope="global" server="JP" />;

export const Searching = () => <LeaderboardToolbar {...base} ranking={{ kind: "score", sort: "total_score" }} catalog={EMPTY_CATALOG} interval="30 days" movementOnly={false} query="Ceylonade" scope="global" server="CN" />;

/** Ranked by an item from the catalog: the picker shows the item. */
export const RankedByItem = () => <LeaderboardToolbar {...base} ranking={{ kind: "item", item: "4002" }} catalog={CATALOG} interval="1 day" movementOnly={false} query="" scope="global" server="EN" />;
