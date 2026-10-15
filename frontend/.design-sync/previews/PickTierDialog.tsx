import { PickTierDialog } from "frontend";
import type { ReactNode } from "react";

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

const wisadel = op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null);
const suzuran = op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa");
const myrtle = op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes");

const TIERS = [
    { id: "tier-s", name: "S", color: "#dc4d56", description: "Warps a map on its own.", entityKeys: ["operator:char_1035_wisdel", "operator:char_4064_mlynar", "operator:char_1028_texas2", "operator:char_4087_ines"] },
    { id: "tier-a", name: "A", color: "#e08a3c", description: "Best-in-slot for most endgame content.", entityKeys: ["operator:char_350_surtr", "operator:char_4116_blkkgt", "operator:char_2012_typhon"] },
    { id: "tier-b", name: "B", color: "#c9a227", description: "Strong, but wants a specific squad.", entityKeys: ["operator:char_103_angel", "operator:char_180_amgoat"] },
    { id: "tier-c", name: "C", color: "#4f9d69", description: "Fine on clear, outclassed at CM.", entityKeys: ["operator:char_263_skadi"] },
];

const noop = () => {};

/** Full-viewport stage: the popup is `position: fixed` against the story root, so a short stage crops it. */
const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-dvh">{children}</div>;

export const PlacedWithNote = () => (
    <Stage>
        <PickTierDialog
            entity={wisadel}
            currentTierId="tier-s"
            description="Free-aim Nuke covers the whole right lane on IS Ashring, and D32 shells delete the elite spawn before it reaches the choke. Only drops if you can't afford the 3-block deployment window."
            tiers={TIERS}
            onClose={noop}
            onPick={noop}
            onDescriptionChange={noop}
        />
    </Stage>
);

export const UnplacedOperator = () => (
    <Stage>
        <PickTierDialog entity={suzuran} currentTierId={null} description="" tiers={TIERS} onClose={noop} onPick={noop} onDescriptionChange={noop} />
    </Stage>
);

/** A support-focused ladder: long tier labels fall back to their initial in the swatch. */
const SUPPORT_TIERS = [
    { id: "tier-core", name: "Core", color: "#dc4d56", description: "Bring one on every squad.", entityKeys: ["operator:char_151_myrtle", "operator:char_128_plosis", "operator:char_358_lisa"] },
    { id: "tier-strong", name: "Strong", color: "#e08a3c", description: "Swap in when the map allows it.", entityKeys: ["operator:char_002_amiya", "operator:char_102_texas"] },
    { id: "tier-niche", name: "Niche", color: "#5a7fb8", description: "Specific stages only.", entityKeys: ["operator:char_199_yak"] },
];

export const LongTierLabels = () => (
    <Stage>
        <PickTierDialog entity={myrtle} currentTierId="tier-core" description="Still the DP benchmark. Every fast-redeploy strategy on Chapter 8 assumes she is on field by second 10." tiers={SUPPORT_TIERS} onClose={noop} onPick={noop} onDescriptionChange={noop} />
    </Stage>
);
