import type { ReactNode } from "react";
import { DragControllerProvider, EditTierRow } from "frontend";

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

const S_PLUS = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
    op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
];

const BUDGET = [op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"), op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"), op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"), op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "rhodes")];

const ALL = [...S_PLUS, ...BUDGET];
const entityByKey = Object.fromEntries(ALL.map((o) => [o.key, o]));

// The editor board owns the sizing custom properties every row reads.
const BOARD_VARS = {
    "--tier-row-min": "90px",
    "--tier-label-h": "100%",
    "--tier-label-w": "108px",
    "--op-size": "58px",
    "--op-gap": "6px",
    "--op-pad-x": "10px",
    "--op-pad-y": "10px",
    "--op-radius": "8px",
} as React.CSSProperties;

const noop = () => {};

const Board = ({ children }: { children: ReactNode }) => (
    <DragControllerProvider entityByKey={entityByKey} onPlace={noop} onUnplace={noop}>
        <div className="overflow-hidden rounded-xl border border-border bg-card shadow-xs" style={BOARD_VARS}>
            {children}
        </div>
    </DragControllerProvider>
);

const rowProps = {
    onMoveUp: noop,
    onMoveDown: noop,
    onOpenSettings: noop,
    onPlace: noop,
    onUnplace: noop,
    onActivateEntity: noop,
};

const sPlusTier = { id: "tier-s-plus", name: "S+", color: "#dc4d56", description: "Solves a risk category alone.", entityKeys: S_PLUS.map((o) => o.key) };
const budgetTier = { id: "tier-b", name: "Budget", color: "#5dbf86", description: "", entityKeys: BUDGET.map((o) => o.key) };
const emptyTier = { id: "tier-c", name: "Situational", color: "#52b9b3", description: "", entityKeys: [] };

const noted = new Set(["operator:char_1035_wisdel", "operator:char_1028_texas2"]);

export const TopRow = () => (
    <Board>
        <EditTierRow tier={sPlusTier} entities={S_PLUS} notedKeys={noted} canMoveUp={false} canMoveDown {...rowProps} />
    </Board>
);

export const StackedRows = () => (
    <Board>
        <EditTierRow tier={sPlusTier} entities={S_PLUS} notedKeys={noted} canMoveUp={false} canMoveDown {...rowProps} />
        <EditTierRow tier={budgetTier} entities={BUDGET} notedKeys={new Set()} canMoveUp canMoveDown {...rowProps} />
    </Board>
);

export const EmptyDropArea = () => (
    <Board>
        <EditTierRow tier={emptyTier} entities={[]} notedKeys={new Set()} canMoveUp canMoveDown={false} {...rowProps} />
    </Board>
);
