import type { ReactNode } from "react";
import { DragControllerProvider, EditTierRow } from "frontend";

const op = (id: string, name: string, rarity: number, profession: string, sub: string, position: string, nationId: string | null) => ({
    id,
    name,
    appellation: null,
    rarity,
    profession,
    subProfessionId: sub,
    position,
    nationId,
    subOrder: 0,
    description: null,
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
const operatorById = Object.fromEntries(ALL.map((o) => [o.id, o]));

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
    <DragControllerProvider operatorById={operatorById} onPlace={noop} onUnplace={noop}>
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
    onActivateOperator: noop,
};

const sPlusTier = { id: "tier-s-plus", name: "S+", color: "#dc4d56", description: "Solves a risk category alone.", operatorIds: S_PLUS.map((o) => o.id) };
const budgetTier = { id: "tier-b", name: "Budget", color: "#5dbf86", description: "", operatorIds: BUDGET.map((o) => o.id) };
const emptyTier = { id: "tier-c", name: "Situational", color: "#52b9b3", description: "", operatorIds: [] };

const noted = new Set(["char_1035_wisdel", "char_1028_texas2"]);

export const TopRow = () => (
    <Board>
        <EditTierRow tier={sPlusTier} operators={S_PLUS} notedOperatorIds={noted} canMoveUp={false} canMoveDown {...rowProps} />
    </Board>
);

export const StackedRows = () => (
    <Board>
        <EditTierRow tier={sPlusTier} operators={S_PLUS} notedOperatorIds={noted} canMoveUp={false} canMoveDown {...rowProps} />
        <EditTierRow tier={budgetTier} operators={BUDGET} notedOperatorIds={new Set()} canMoveUp canMoveDown {...rowProps} />
    </Board>
);

export const EmptyDropArea = () => (
    <Board>
        <EditTierRow tier={emptyTier} operators={[]} notedOperatorIds={new Set()} canMoveUp canMoveDown={false} {...rowProps} />
    </Board>
);
