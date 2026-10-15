import { RankByPicker } from "frontend";
import { type ReactNode, useEffect, useRef, useState } from "react";

// RankByPicker chooses what the user leaderboard ranks by: a score (total,
// operators, stages, ...) or any inventory item. `toolbar` is the labelled
// "RANK BY" control in the filter row (with the item's icon when an item
// ranks); `header` is the compact uppercase trigger over the table's value
// column. Both open one popover: a search, the score sorts, then the featured
// inventory items (currencies first), each with its holder count and how much
// is held in all. Holder counts and totals are measured from the synced
// population (2,630 players; Originite Prime has only 2 holders because it is
// stored only for players who synced after 2026-09-15).

type Item = { item_id: string; holders: number; top: number; total_quantity: number; meta: null; name: string; rarityNum: number; iconId: string };
const it = (item_id: string, name: string, rarityNum: number, iconId: string, holders: number, top: number, total_quantity: number): Item => ({ item_id, holders, top, total_quantity, meta: null, name, rarityNum, iconId });

const CATALOG = {
    population: 2630,
    items: [
        it("3401", "Furniture Part", 3, "COIN_FURN", 2343, 230072, 44575337),
        it("4006", "Purchase Certificate", 3, "EXGG_SHD", 2340, 24520, 3384593),
        it("4001", "LMD", 4, "GOLD", 2630, 115648731, 5204086400),
        it("4003", "Orundum", 5, "DIAMOND_SHD", 2625, 943200, 90009224),
        it("4004", "Distinction Certificate", 5, "HGG_SHD", 2621, 9834, 394423),
        it("4005", "Commendation Certificate", 3, "LGG_SHD", 2352, 96633, 12220953),
        it("SOCIAL_PT", "Credit", 2, "SOCIAL_PT", 2318, 1447, 489676),
        it("mod_unlock_token", "Module Data Block", 5, "mod_unlock_token", 2308, 883, 134117),
        it("3003", "Pure Gold", 4, "MTL_GOLD3", 2247, 96191, 3541781),
        it("7003", "Headhunting Permit", 5, "TKT_GACHA", 1614, 277, 15901),
        it("7004", "Ten-roll Headhunting Permit", 5, "TKT_GACHA_10", 579, 98, 1668),
        it("4002", "Originite Prime", 6, "DIAMOND", 2, 899, 922),
    ],
};

type Ranking = { kind: "score"; sort: "total_score" } | { kind: "item"; item: string };

function Picker({ initial, variant }: { initial: Ranking; variant: "toolbar" | "header" }) {
    const [ranking, setRanking] = useState<Ranking>(initial);
    return <RankByPicker ranking={ranking} onRanking={(next) => setRanking(next as Ranking)} catalog={CATALOG} materials={undefined} variant={variant} />;
}

/** Clicks the trigger after first paint (Base UI wires it late), then drops the autofocus ring off the search. */
function OpenOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const timers: number[] = [];
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLButtonElement>("button")?.click();
                for (const ms of [60, 180, 400]) timers.push(window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            for (const t of timers) clearTimeout(t);
        };
    }, []);
    return (
        <div ref={ref} style={{ minHeight: 620 }} className="relative w-full">
            {children}
        </div>
    );
}

/** The toolbar control ranking by total score. */
export const ToolbarScore = () => <Picker initial={{ kind: "score", sort: "total_score" }} variant="toolbar" />;

/** The toolbar control ranking an item: its icon leads. */
export const ToolbarItem = () => <Picker initial={{ kind: "item", item: "3003" }} variant="toolbar" />;

/** The compact header trigger over the value column. */
export const HeaderTrigger = () => (
    <div style={{ width: 420 }} className="flex items-center justify-end border-b px-3 py-2">
        <Picker initial={{ kind: "item", item: "4003" }} variant="header" />
    </div>
);

/** Open from the toolbar: search, the score sorts, then the featured inventory items with holder counts. */
export const Open = () => (
    <OpenOnMount>
        <Picker initial={{ kind: "item", item: "4003" }} variant="toolbar" />
    </OpenOnMount>
);
