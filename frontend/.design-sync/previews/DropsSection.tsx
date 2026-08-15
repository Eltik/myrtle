import { DropsSection } from "frontend";

/** `groupDrops(stage, materials)` for 4-10 — Extinguished Flames. */
const GROUPS = [
    {
        type: "COMPLETE",
        label: "On Completion",
        order: 1,
        drops: [
            {
                reward: {dropType: "COMPLETE", id: "4002", occPercent: "ALWAYS", itemType: "DIAMOND"},
                name: "Originite Prime",
                iconUrl: "https://api.myrtle.moe/api/item-icon/DIAMOND",
                rarity: 6,
                isChar: false,
                occ: {label: "Guaranteed", level: 5, tone: "var(--success)"},
            },
        ],
    },
    {
        type: "NORMAL",
        label: "Regular Drops",
        order: 2,
        drops: [
            {
                reward: {dropType: "NORMAL", id: "30063", occPercent: "OFTEN", itemType: "MATERIAL"},
                name: "Integrated Device",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_BOSS3",
                rarity: 3,
                isChar: false,
                occ: {label: "Rare", level: 2, tone: "var(--info)"},
            },
        ],
    },
    {
        type: "SPECIAL",
        label: "Special Drops",
        order: 3,
        drops: [
            {
                reward: {dropType: "SPECIAL", id: "30064", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Optimized Device",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_BOSS4",
                rarity: 4,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
        ],
    },
    {
        type: "ADDITIONAL",
        label: "Bonus Drops",
        order: 4,
        drops: [
            {
                reward: {dropType: "ADDITIONAL", id: "30012", occPercent: "OFTEN", itemType: "MATERIAL"},
                name: "Orirock Cube",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_G2",
                rarity: 2,
                isChar: false,
                occ: {label: "Rare", level: 2, tone: "var(--info)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30011", occPercent: "OFTEN", itemType: "MATERIAL"},
                name: "Orirock",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_G1",
                rarity: 1,
                isChar: false,
                occ: {label: "Rare", level: 2, tone: "var(--info)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "3003", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Pure Gold",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_GOLD3",
                rarity: 4,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30013", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Orirock Cluster",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_G3",
                rarity: 3,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30063", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Integrated Device",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_BOSS3",
                rarity: 3,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30073", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Loxic Kohl",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_ALCOHOL1",
                rarity: 3,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "31013", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Coagulating Gel",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_PGEL3",
                rarity: 3,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "31073", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Fuscous Fiber",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_XW",
                rarity: 3,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30062", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Device",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_BOSS2",
                rarity: 2,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
            {
                reward: {dropType: "ADDITIONAL", id: "30061", occPercent: "SOMETIMES", itemType: "MATERIAL"},
                name: "Damaged Device",
                iconUrl: "https://api.myrtle.moe/api/item-icon/MTL_SL_BOSS1",
                rarity: 1,
                isChar: false,
                occ: {label: "Very Rare", level: 1, tone: "var(--warning)"},
            },
        ],
    },
];

const byType = (type: string) => GROUPS.filter((g) => g.type === type);

/** All four buckets. The Bonus group is cut to its first six of ten so the page fits one screen. */
export const StageDrops = () => (
    <div className="max-w-2xl">
        <DropsSection groups={GROUPS.map((g) => (g.type === "ADDITIONAL" ? { ...g, drops: g.drops.slice(0, 6) } : g))} />
    </div>
);

/** The two buckets a farming run cares about. */
export const RegularAndSpecial = () => (
    <div className="max-w-2xl">
        <DropsSection groups={[...byType("NORMAL"), ...byType("SPECIAL")]} />
    </div>
);

/** First-clear rewards sit in their own group, above everything else. */
export const FirstClearOnly = () => (
    <div className="max-w-2xl">
        <DropsSection groups={byType("COMPLETE")} />
    </div>
);

/** The rate meter fills 0-5 bars per band, from Guaranteed down to Very Rare. */
export const RateBands = () => (
    <div className="max-w-2xl">
        <DropsSection
            groups={[
                {
                    type: "NORMAL",
                    label: "Drop Rates",
                    order: 0,
                    drops: [
                        { ...byType("COMPLETE")[0].drops[0], occ: { label: "Guaranteed", level: 5, tone: "var(--success)" } },
                        { ...byType("ADDITIONAL")[0].drops[0], occ: { label: "Common", level: 4, tone: "var(--success)" } },
                        { ...byType("ADDITIONAL")[0].drops[1], occ: { label: "Uncommon", level: 3, tone: "var(--info)" } },
                        { ...byType("NORMAL")[0].drops[0], occ: { label: "Rare", level: 2, tone: "var(--info)" } },
                        { ...byType("SPECIAL")[0].drops[0], occ: { label: "Very Rare", level: 1, tone: "var(--warning)" } },
                        { ...byType("ADDITIONAL")[0].drops[2], occ: { label: "Very Rare", level: 0, tone: "var(--destructive)" } },
                    ],
                },
            ]}
        />
    </div>
);
