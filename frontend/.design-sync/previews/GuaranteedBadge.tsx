import { GuaranteedBadge } from "frontend";

// The worst-case rarity of a recruitment tag combination, shown on every
// result because the list is ranked by it. A 5★/6★ lock is a filled,
// rarity-tinted badge ("5★ guaranteed"); anything lower is a quiet "Min" label
// with colour only on the value. `layout` only changes alignment.
//
// The badge reads the floor off `guaranteedRarity` and the pool's rarities, so
// each fixture carries its real pool (calculateResults over the EN gacha table).

const op = (id: string, name: string, rarity: number, profession: string, professionName: string, tagList: string[]) => ({ id, name, rarity, profession, professionName, position: tagList[0] === "Melee" ? "MELEE" : "RANGED", tagList, potentials: [] });

const topSniper = {
    tags: [11, 2],
    tagNames: ["Top Operator", "Sniper"],
    guaranteedRarity: 6,
    maxRarity: 6,
    fiveStarCount: 4,
    operators: [op("char_103_angel", "Exusiai", 6, "SNIPER", "Sniper", ["Ranged", "Sniper", "Top Operator", "DPS"])],
};

const seniorMedic = {
    tags: [14, 4],
    tagNames: ["Senior Operator", "Medic"],
    guaranteedRarity: 5,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "Medic", ["Ranged", "Medic", "Senior Operator", "Healing", "Support"])],
};

const slowRanged = {
    tags: [23, 10],
    tagNames: ["Slow", "Ranged"],
    guaranteedRarity: 3,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [op("char_195_glassb", "Istina", 5, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Senior Operator", "Slow", "DPS"]), op("char_278_orchid", "Orchid", 3, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Slow"])],
};

const robot = {
    tags: [28],
    tagNames: ["Robot"],
    guaranteedRarity: 1,
    maxRarity: 1,
    fiveStarCount: 0,
    operators: [op("char_285_medic2", "Lancet-2", 1, "MEDIC", "Medic", ["Ranged", "Medic", "Robot", "Healing"])],
};

/** A Top Operator lock: the filled 6★ badge. */
export const SixStarLock = () => <GuaranteedBadge result={topSniper} layout="compact" />;

/** A Senior Operator lock: the filled 5★ badge. */
export const FiveStarLock = () => <GuaranteedBadge result={seniorMedic} layout="compact" />;

/** No lock: the quiet "Min 3★" label. */
export const MinThreeStar = () => <GuaranteedBadge result={slowRanged} layout="compact" />;

/** The Robot tag floors at the robot tier, which reads "Robot" rather than "1★". */
export const RobotFloor = () => <GuaranteedBadge result={robot} layout="compact" />;

/** In a detailed card header the quiet label pushes to the right edge. */
export const DetailedHeader = () => (
    <div className="flex w-96 items-center gap-1.5 rounded-lg border border-border bg-card px-4 py-3">
        <span className="rounded-sm border border-border px-1.5 text-xs">Slow</span>
        <span className="rounded-sm border border-border px-1.5 text-xs">Ranged</span>
        <GuaranteedBadge result={slowRanged} layout="detailed" />
    </div>
);
