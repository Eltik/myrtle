import type { ReactNode } from "react";
import { TierRow } from "frontend";

/**
 * An operator placement as the API maps it (`toTierEntity`): a resolved
 * `operator` entity keyed `operator:<id>`, icon an API path. Every id verified
 * against https://api.myrtle.moe/api/operators/index.
 */
const PROFESSION_NAME: Record<string, string> = { PIONEER: "Vanguard", WARRIOR: "Guard", TANK: "Defender", SNIPER: "Sniper", CASTER: "Caster", MEDIC: "Medic", SUPPORT: "Supporter", SPECIAL: "Specialist" };
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string | null, description: string | null = null) => ({
    key: `operator:${id}`,
    kind: "operator" as const,
    id,
    name,
    icon: `/avatar/${id}`,
    href: `/operators/${id}`,
    facets: {},
    resolved: true as const,
    appellation: null,
    rarity: rarity as 1 | 2 | 3 | 4 | 5 | 6,
    profession: profession as never,
    subProfessionId,
    professionName: PROFESSION_NAME[profession] ?? null,
    subProfessionName: null,
    position: position as never,
    nationId: nationId || null,
    nationName: null,
    subOrder: 0,
    description,
    updatedAt: "2024-05-12T14:05:00.000Z",
});

// `TierRow` is an <li> sized by the custom properties `TierListBoard` sets on the
// board <ul>. Reproduce that surface so a standalone row measures the same way it
// does on the detail page.
const BOARD_VARS = {
    "--tier-row-min": "92px",
    "--tier-label-w": "124px",
    "--op-size": "64px",
    "--op-gap": "6px",
    "--op-pad-x": "12px",
    "--op-pad-y": "10px",
    "--op-radius": "8px",
} as React.CSSProperties;

const Board = ({ children }: { children: ReactNode }) => (
    <ul className="m-0 list-none overflow-hidden rounded-xl border border-border bg-card p-0 shadow-xs" style={BOARD_VARS}>
        {children}
    </ul>
);

const topTier = {
    id: "tier-s-plus",
    name: "S+",
    displayOrder: 0,
    color: "#dc4d56",
    description: "Solves a whole risk category on their own. Bring them unless the stage locks them out.",
    entities: [
        op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null, "Deletes the Ritualist wave from off-screen."),
        op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
        op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
        op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
        op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
    ],
};

const midTier = {
    id: "tier-s",
    name: "S",
    displayOrder: 1,
    color: "#e0834a",
    description: null,
    entities: [
        op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"),
        op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"),
        op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa"),
        op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "leithanien"),
    ],
};

const budgetTier = {
    id: "tier-b",
    name: "B",
    displayOrder: 2,
    color: "#5dbf86",
    description: "Budget answers worth raising if the top of the list is out of reach.",
    entities: [op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"), op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"), op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"), op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "rhodes")],
};

const longNameTier = {
    id: "tier-situational",
    name: "Situational picks",
    displayOrder: 3,
    color: null,
    description: "Only worth a slot when the map has a Ranged Restriction lane.",
    entities: [op("char_474_glady", "Gladiia", 6, "SPECIAL", "hookmaster", "MELEE", "egir"), op("char_400_weedy", "Weedy", 6, "SPECIAL", "pusher", "MELEE", "rhodes"), op("char_4048_doroth", "Dorothy", 6, "SPECIAL", "traper", "RANGED", "columbia")],
};

const emptyTier = {
    id: "tier-c",
    name: "C",
    displayOrder: 4,
    color: "#8a8a8a",
    description: "Nothing has fallen this far since the module pass.",
    entities: [],
};

export const TopTier = () => (
    <Board>
        <TierRow tier={topTier} index={0} />
    </Board>
);

export const StackedRows = () => (
    <Board>
        <TierRow tier={topTier} index={0} />
        <TierRow tier={midTier} index={1} />
        <TierRow tier={budgetTier} index={2} />
    </Board>
);

export const LongTierName = () => (
    <Board>
        <TierRow tier={longNameTier} index={3} />
    </Board>
);

export const EmptyTier = () => (
    <Board>
        <TierRow tier={emptyTier} index={4} />
    </Board>
);
