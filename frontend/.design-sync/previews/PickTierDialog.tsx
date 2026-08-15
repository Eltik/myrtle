import { PickTierDialog } from "frontend";
import type { ReactNode } from "react";

/** `ITierOperator` — ids verified against https://api.myrtle.moe/api/operators/index. */
const wisadel = {
    id: "char_1035_wisdel",
    name: "Wiš'adel",
    appellation: null,
    rarity: 6,
    profession: "SNIPER",
    subProfessionId: "bombarder",
    position: "RANGED",
    nationId: null,
    subOrder: 0,
    description: null,
    updatedAt: "2024-05-14T18:02:00.000Z",
};

const suzuran = { ...wisadel, id: "char_358_lisa", name: "Suzuran", profession: "SUPPORT", subProfessionId: "slower", nationId: "siracusa" };

const myrtle = { ...wisadel, id: "char_151_myrtle", name: "Myrtle", rarity: 4, profession: "PIONEER", subProfessionId: "bearer", position: "MELEE", nationId: "rhodes" };

const TIERS = [
    { id: "tier-s", name: "S", color: "#dc4d56", description: "Warps a map on its own.", operatorIds: ["char_1035_wisdel", "char_4064_mlynar", "char_1028_texas2", "char_4087_ines"] },
    { id: "tier-a", name: "A", color: "#e08a3c", description: "Best-in-slot for most endgame content.", operatorIds: ["char_350_surtr", "char_4116_blkkgt", "char_2012_typhon"] },
    { id: "tier-b", name: "B", color: "#c9a227", description: "Strong, but wants a specific squad.", operatorIds: ["char_103_angel", "char_180_amgoat"] },
    { id: "tier-c", name: "C", color: "#4f9d69", description: "Fine on clear, outclassed at CM.", operatorIds: ["char_263_skadi"] },
];

const noop = () => {};

/** Full-viewport stage: the popup is `position: fixed` against the story root, so a short stage crops it. */
const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-dvh">{children}</div>;

export const PlacedWithNote = () => (
    <Stage>
        <PickTierDialog
            operator={wisadel}
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
        <PickTierDialog operator={suzuran} currentTierId={null} description="" tiers={TIERS} onClose={noop} onPick={noop} onDescriptionChange={noop} />
    </Stage>
);

/** A support-focused ladder: long tier labels fall back to their initial in the swatch. */
const SUPPORT_TIERS = [
    { id: "tier-core", name: "Core", color: "#dc4d56", description: "Bring one on every squad.", operatorIds: ["char_151_myrtle", "char_128_plosis", "char_358_lisa"] },
    { id: "tier-strong", name: "Strong", color: "#e08a3c", description: "Swap in when the map allows it.", operatorIds: ["char_002_amiya", "char_102_texas"] },
    { id: "tier-niche", name: "Niche", color: "#5a7fb8", description: "Specific stages only.", operatorIds: ["char_199_yak"] },
];

export const LongTierLabels = () => (
    <Stage>
        <PickTierDialog operator={myrtle} currentTierId="tier-core" description="Still the DP benchmark. Every fast-redeploy strategy on Chapter 8 assumes she is on field by second 10." tiers={SUPPORT_TIERS} onClose={noop} onPick={noop} onDescriptionChange={noop} />
    </Stage>
);
