import type { ReactNode } from "react";
import { TierOperatorPreview } from "frontend";

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

// On the detail page this card is the body of the hover card that opens off an
// operator tile, so the popover surface belongs to `HoverCardContent`.
const Popover = ({ children }: { children: ReactNode }) => <div className="w-max overflow-hidden rounded-lg border border-border bg-popover shadow-md">{children}</div>;

export const SixStarWithNote = () => (
    <Popover>
        <TierOperatorPreview operator={op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null, "Deletes the Ritualist wave from off-screen before it reaches the choke. S3 only — S2 leaves one caster alive at Risk 18.")} />
    </Popover>
);

export const AliasName = () => (
    <Popover>
        <TierOperatorPreview operator={op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen", "Covers the left lane on her own once the second Sarkaz group spawns.")} />
    </Popover>
);

export const NoPlacementNote = () => (
    <Popover>
        <TierOperatorPreview operator={op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "leithanien")} />
    </Popover>
);

export const LowRarity = () => (
    <Popover>
        <TierOperatorPreview operator={op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes", "Still the cheapest DP engine in the game. Raise her before any other Vanguard.")} />
    </Popover>
);
