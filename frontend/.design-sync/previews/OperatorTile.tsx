import type { ReactNode } from "react";
import { OperatorTile } from "frontend";

const op = (id: string, name: string, rarity: number, profession: string, sub: string, position: string, nationId: string | null, description: string | null = null) => ({
    id,
    name,
    appellation: null,
    rarity,
    profession,
    subProfessionId: sub,
    position,
    nationId,
    subOrder: 0,
    description,
    updatedAt: "2024-05-12T14:05:00.000Z",
});

// The tile sizes itself from the custom properties the tier board sets, so a
// standalone preview has to supply them the same way `TierListBoard` does.
const TILE_VARS = { "--op-size": "64px", "--op-radius": "8px" } as React.CSSProperties;

const Strip = ({ label, children }: { label: string; children: ReactNode }) => (
    <div className="w-fit max-w-xl rounded-xl border border-border bg-card p-3" style={TILE_VARS}>
        <p className="m-0 mb-2 font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{label}</p>
        <div className="flex flex-wrap items-start gap-1.5">{children}</div>
    </div>
);

const SIX_STARS = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null, "Deletes the Ritualist wave from off-screen. S3 only — S2 is a trap at Risk 18."),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
    op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
];

const RARITY_LADDER = [
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "rhodes"),
];

export const TopTierRow = () => (
    <Strip label="Tier S+">
        {SIX_STARS.map((o) => (
            <OperatorTile key={o.id} operator={o} />
        ))}
    </Strip>
);

export const RarityLadder = () => (
    <Strip label="Rarity accent runs along the bottom edge">
        {RARITY_LADDER.map((o) => (
            <OperatorTile key={o.id} operator={o} />
        ))}
    </Strip>
);

const FULL_TIER = [
    ...SIX_STARS,
    op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"),
    op("char_4039_horn", "Horn", 6, "TANK", "fortress", "MELEE", "victoria"),
    op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa"),
    op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer", "RANGED", null),
    op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "victoria"),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "lungmen"),
];

export const WrappingTier = () => (
    <Strip label="Tier S — wraps onto a second line">
        {FULL_TIER.map((o) => (
            <OperatorTile key={o.id} operator={o} />
        ))}
    </Strip>
);
