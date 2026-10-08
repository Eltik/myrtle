import { DragControllerProvider, EditTierRow, EntityPool } from "frontend";
import type { CSSProperties, ReactNode } from "react";

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

const entityByKey: Record<string, (typeof ROSTER)[number]> = Object.fromEntries(ROSTER.map((entry) => [entry.key, entry]));

const TIERS = [
    { id: "tier-s", name: "S", color: "#dc4d56", description: "Warps a map on its own.", entityKeys: ["operator:char_1035_wisdel", "operator:char_4064_mlynar", "operator:char_1028_texas2", "operator:char_4087_ines"] },
    { id: "tier-a", name: "A", color: "#e0603c", description: "Best-in-slot for most endgame content.", entityKeys: ["operator:char_350_surtr", "operator:char_4116_blkkgt", "operator:char_2012_typhon", "operator:char_377_gdglow", "operator:char_003_kalts"] },
    { id: "tier-b", name: "B", color: "#c9a227", description: "Strong, but wants a specific squad.", entityKeys: ["operator:char_103_angel", "operator:char_180_amgoat", "operator:char_293_thorns", "operator:char_179_cgbird"] },
    { id: "tier-c", name: "C", color: "#4f9d69", description: "Fine on clear, outclassed at CM.", entityKeys: ["operator:char_263_skadi", "operator:char_017_huang"] },
];

const NOTED = new Set(["operator:char_1035_wisdel", "operator:char_4087_ines", "operator:char_180_amgoat"]);

const noop = () => {};

/** What the first three tiers hold, so the pool dims them. */
const PLACED = new Set(TIERS.slice(0, 3).flatMap((t) => t.entityKeys));

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
            entities={tier.entityKeys.map((key) => entityByKey[key])}
            notedKeys={NOTED}
            canMoveUp={idx > 0}
            canMoveDown={idx < tiers.length - 1}
            onMoveUp={noop}
            onMoveDown={noop}
            onOpenSettings={noop}
            onPlace={noop}
            onUnplace={noop}
            onActivateEntity={noop}
        />
    ));

export const EditorBoard = () => (
    <DragControllerProvider entityByKey={entityByKey} onPlace={noop} onUnplace={noop}>
        <Board>{rows(TIERS)}</Board>
    </DragControllerProvider>
);

export const BoardWithPool = () => (
    <DragControllerProvider entityByKey={entityByKey} onPlace={noop} onUnplace={noop}>
        <div className="flex items-start gap-4">
            <div className="min-w-0 flex-1">
                <Board>{rows(TIERS.slice(0, 3))}</Board>
            </div>
            <div className="w-80 shrink-0">
                <EntityPool kinds={["operator"]} catalogues={{ operator: { entities: ROSTER, status: "success", refetch: noop } }} placedKeys={PLACED} placedByKind={{ operator: PLACED.size }} onUnplace={noop} onPickerActivate={noop} rootClassName="h-96" />
            </div>
        </div>
    </DragControllerProvider>
);
