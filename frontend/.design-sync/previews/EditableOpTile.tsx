import type { ReactNode } from "react";
import { DragControllerProvider, EditableOpTile } from "frontend";

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

const POOL = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
    op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "rhodes"),
];

const entityByKey = Object.fromEntries(POOL.map((o) => [o.key, o]));

// Chips read their size from the editor board / operator pool custom properties,
// and every chip has to sit inside the drag controller the editor mounts.
const TILE_VARS = { "--op-size": "58px", "--op-radius": "8px" } as React.CSSProperties;

const noop = () => {};

const Stage = ({ label, children }: { label: string; children: ReactNode }) => (
    <DragControllerProvider entityByKey={entityByKey} onPlace={noop} onUnplace={noop}>
        <div className="w-fit max-w-xl rounded-xl border border-border bg-card p-3" style={TILE_VARS}>
            <p className="m-0 mb-2 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{label}</p>
            <div className="flex flex-wrap items-start gap-1.5">{children}</div>
        </div>
    </DragControllerProvider>
);

export const OperatorPoolRow = () => (
    <Stage label="Operator pool — drag onto a tier">
        {POOL.map((o) => (
            <EditableOpTile key={o.id} entity={o} />
        ))}
    </Stage>
);

export const WithPlacementNotes = () => (
    <Stage label="Tier S+ — the dot marks a written placement note">
        {POOL.slice(0, 5).map((o, i) => (
            <EditableOpTile key={o.id} entity={o} hasNote={i < 2} />
        ))}
    </Stage>
);

export const AlreadyPlaced = () => (
    <Stage label="Pool with placed operators dimmed out">
        {POOL.slice(0, 6).map((o, i) => (
            <EditableOpTile key={o.id} entity={o} disabled={i < 3} placed={i < 3} />
        ))}
    </Stage>
);
