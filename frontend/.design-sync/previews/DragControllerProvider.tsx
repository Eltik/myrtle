import { DragControllerProvider, EditTierRow, OperatorPool } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/** `ITierOperator` rows — every id verified against https://api.myrtle.moe/api/operators/index. */
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string) => ({
    id,
    name,
    appellation: "",
    rarity,
    profession,
    subProfessionId,
    position,
    nationId,
    isNotObtainable: false,
    subOrder: 0,
    description: null,
    updatedAt: "2024-05-13T21:40:00.000Z",
});

const ROSTER = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", ""),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    op("char_4087_ines", "Ines", 6, "PIONEER", "agent", "MELEE", ""),
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
    op("char_2012_typhon", "Typhon", 6, "SNIPER", "siegesniper", "RANGED", "sami"),
    op("char_377_gdglow", "Goldenglow", 6, "CASTER", "funnel", "RANGED", "victoria"),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "lungmen"),
    op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "leithanien"),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord", "MELEE", "iberia"),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "MELEE", "egir"),
    op("char_017_huang", "Blaze", 6, "WARRIOR", "centurion", "MELEE", "rhodes"),
    op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa"),
    op("char_003_kalts", "Kal'tsit", 6, "MEDIC", "physician", "RANGED", "rhodes"),
    op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer", "RANGED", ""),
    op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"),
    op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "victoria"),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"),
    op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "siracusa"),
    op("char_143_ghost", "Specter", 5, "WARRIOR", "centurion", "MELEE", "egir"),
    op("char_199_yak", "Matterhorn", 4, "TANK", "protector", "MELEE", "kjerag"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"),
];

const operatorById: Record<string, (typeof ROSTER)[number]> = Object.fromEntries(ROSTER.map((entry) => [entry.id, entry]));

const TIERS = [
    { id: "tier-s", name: "S", color: "#dc4d56", description: "Warps a map on its own.", operatorIds: ["char_1035_wisdel", "char_4064_mlynar", "char_1028_texas2", "char_4087_ines"] },
    { id: "tier-a", name: "A", color: "#e0603c", description: "Best-in-slot for most endgame content.", operatorIds: ["char_350_surtr", "char_4116_blkkgt", "char_2012_typhon", "char_377_gdglow", "char_003_kalts"] },
    { id: "tier-b", name: "B", color: "#c9a227", description: "Strong, but wants a specific squad.", operatorIds: ["char_103_angel", "char_180_amgoat", "char_293_thorns", "char_179_cgbird"] },
    { id: "tier-c", name: "C", color: "#4f9d69", description: "Fine on clear, outclassed at CM.", operatorIds: ["char_263_skadi", "char_017_huang"] },
];

const NOTED = new Set(["char_1035_wisdel", "char_4087_ines", "char_180_amgoat"]);

const noop = () => {};

/** The sizing tokens `Editor.module.css` puts on `.board`, at its ≥640px step. */
const boardVars = {
    "--tier-row-min": "90px",
    "--tier-label-h": "100%",
    "--tier-label-w": "108px",
    "--op-size": "58px",
    "--op-gap": "6px",
    "--op-pad-x": "10px",
    "--op-pad-y": "10px",
    "--op-radius": "8px",
} as CSSProperties;

const Board = ({ children }: { children?: ReactNode }) => (
    <section className="overflow-hidden rounded-xl border border-border bg-card shadow-[0_1px_2px_oklch(0_0_0/0.04)]" style={boardVars} aria-label="Edit board for Endgame DPS rankings">
        {children}
    </section>
);

const rows = (tiers: typeof TIERS) =>
    tiers.map((tier, idx) => (
        <EditTierRow
            key={tier.id}
            tier={tier}
            operators={tier.operatorIds.map((id) => operatorById[id])}
            notedOperatorIds={NOTED}
            canMoveUp={idx > 0}
            canMoveDown={idx < tiers.length - 1}
            onMoveUp={noop}
            onMoveDown={noop}
            onOpenSettings={noop}
            onPlace={noop}
            onUnplace={noop}
            onActivateOperator={noop}
        />
    ));

export const EditorBoard = () => (
    <DragControllerProvider operatorById={operatorById} onPlace={noop} onUnplace={noop}>
        <Board>{rows(TIERS)}</Board>
    </DragControllerProvider>
);

export const BoardWithPool = () => (
    <DragControllerProvider operatorById={operatorById} onPlace={noop} onUnplace={noop}>
        <div className="flex items-start gap-4">
            <div className="min-w-0 flex-1">
                <Board>{rows(TIERS.slice(0, 3))}</Board>
            </div>
            <div className="w-80 shrink-0">
                <OperatorPool operators={ROSTER} placedIds={new Set(TIERS.flatMap((t) => t.operatorIds))} onUnplace={noop} onPickerActivate={noop} rootClassName="h-96" />
            </div>
        </div>
    </DragControllerProvider>
);
