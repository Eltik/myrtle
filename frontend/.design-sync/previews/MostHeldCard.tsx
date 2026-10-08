import { MostHeldCard } from "frontend";
import { useState } from "react";

// MostHeldCard is the inventory leaderboard's sidebar list: the ten items the
// most visible players hold, each with its icon, name, the largest single
// holding ("Top ...") and how many players hold it. The ranked item is tinted
// with its name in the primary; clicking another re-ranks the board. While the
// catalogue loads it shows six skeleton rows. Figures are measured from the
// synced population (holders, max and sum per item); names and icon ids are
// the EN item_table's.

type Catalog = { item_id: string; holders: number; top: number; total_quantity: number; meta: null; name: string; rarityNum: number; iconId: string };

const row = (item_id: string, name: string, rarityNum: number, iconId: string, holders: number, top: number, total_quantity: number): Catalog => ({ item_id, holders, top, total_quantity, meta: null, name, rarityNum, iconId });

const CATALOG: Catalog[] = [
    row("4006", "Purchase Certificate", 3, "EXGG_SHD", 2340, 24520, 3384593),
    row("mod_unlock_token", "Module Data Block", 5, "mod_unlock_token", 2308, 883, 134117),
    row("30012", "Orirock Cube", 2, "MTL_SL_G2", 2280, 7372, 822710),
    row("mod_update_token_2", "Data Supplement Instrument", 5, "mod_update_token_2", 2254, 1963, 341331),
    row("3003", "Pure Gold", 4, "MTL_GOLD3", 2247, 96191, 3541781),
    row("mod_update_token_1", "Data Supplement Stick", 4, "mod_update_token_1", 2245, 6540, 1286134),
    row("30103", "RMA70-12", 3, "MTL_SL_RMA7012", 2217, 1204, 189200),
    row("31043", "Semi-Synthetic Solvent", 3, "MTL_SL_SS", 2211, 953, 220805),
    row("31083", "Aggregate Cyclicene", 3, "MTL_SL_HT", 2189, 972, 186123),
    row("3303", "Skill Summary - 3", 4, "MTL_SKILL3", 2180, 3518, 160261),
    row("30083", "Manganese Ore", 3, "MTL_SL_MANGANESE1", 2165, 1612, 170226),
];

function Stage({ initial, isLoading = false, catalog = CATALOG }: { initial: string; isLoading?: boolean; catalog?: Catalog[] }) {
    const [current, setCurrent] = useState(initial);
    return (
        <div style={{ width: 300 }}>
            <MostHeldCard catalog={catalog} current={current} onItem={setCurrent} isLoading={isLoading} />
        </div>
    );
}

/** Ranking Pure Gold: its row tinted, the top ten listed. */
export const RankingPureGold = () => <Stage initial="3003" />;

/** Ranking the most-held item, the first row. */
export const RankingFirst = () => <Stage initial="4006" />;

/** Before the catalogue loads: six skeleton rows. */
export const Loading = () => <Stage initial="3003" isLoading catalog={[]} />;
