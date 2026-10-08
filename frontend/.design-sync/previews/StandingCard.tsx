import { StandingCard } from "frontend";

// StandingCard is the signed-in player's place on the inventory leaderboard
// for the ranked item: rank pill, avatar and nickname with "Rank #n · top x%",
// then three metrics (how many they hold with the item's icon, their rank on
// their own server, and how many players hold it) and a View profile button.
// A player who holds none gets a line instead of the metrics. It renders
// nothing while loading. Figures sit inside the measured population (2,247
// Pure Gold holders, 2,308 Module Data Block holders).

const PURE_GOLD = { item_id: "3003", holders: 2247, top: 96191, total_quantity: 3541781, meta: null, name: "Pure Gold", rarityNum: 4, iconId: "MTL_GOLD3" };
const MODULE_BLOCK = { item_id: "mod_unlock_token", holders: 2308, top: 883, total_quantity: 134117, meta: null, name: "Module Data Block", rarityNum: 5, iconId: "mod_unlock_token" };

const ME = { uid: "4815162342", server: "en", nickname: "Ashlock", avatar_id: "char_291_aglina" };

const Frame = ({ children }: { children: React.ReactNode }) => <div style={{ width: 300 }}>{children}</div>;

/** Holding 2,412 Pure Gold: #354 of 2,247 holders. */
export const MidTable = () => (
    <Frame>
        <StandingCard item={PURE_GOLD} me={ME} standing={{ item_id: "3003", quantity: 2412, rank_global: 354, holders_global: 2247, rank_server: 198, holders_server: 1310 }} />
    </Frame>
);

/** Near the top: 212 Module Data Blocks, #69. */
export const NearTheTop = () => (
    <Frame>
        <StandingCard item={MODULE_BLOCK} me={ME} standing={{ item_id: "mod_unlock_token", quantity: 212, rank_global: 69, holders_global: 2308, rank_server: 37, holders_server: 1342 }} />
    </Frame>
);

/** Holds none of the item: no rank, a line in place of the metrics. */
export const HoldsNone = () => (
    <Frame>
        <StandingCard item={PURE_GOLD} me={{ ...ME, nickname: null, avatar_id: null }} standing={null} />
    </Frame>
);
