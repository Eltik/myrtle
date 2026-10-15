import { Dialog, DialogPopup, TierDetailsDialog } from "frontend";

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

const topTier = {
    id: "tier-s-plus",
    name: "S+",
    displayOrder: 0,
    color: "#dc4d56",
    description: "Operators that solve a whole risk category on their own. Everything here clears Risk 18 **without** a second slot spent on support, which is why the bar is so much higher than S.",
    entities: [
        op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", null, "Deletes the Ritualist wave from off-screen before it reaches the choke. S3 only — S2 leaves one caster alive."),
        op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz", "Holds the left lane with no healer attached, which frees the slot that used to go to Nightingale."),
        op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
        op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    ],
};

const plainTier = {
    id: "tier-a",
    name: "A",
    displayOrder: 2,
    color: "#d8b54a",
    description: null,
    entities: [op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "victoria"), op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "lungmen"), op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"), op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes")],
};

const emptyTier = {
    id: "tier-c",
    name: "Situational picks",
    displayOrder: 4,
    color: "#52b9b3",
    description: "Reserved for operators that only earn a slot on maps with a Ranged Restriction lane. Nothing has been placed here since the module pass.",
    entities: [],
};

export const TopTier = () => (
    <Dialog open>
        <DialogPopup className="max-w-2xl overflow-hidden p-0" initialFocus={false}>
            <TierDetailsDialog tier={topTier} color="#dc4d56" />
        </DialogPopup>
    </Dialog>
);

export const NoDescription = () => (
    <Dialog open>
        <DialogPopup className="max-w-2xl overflow-hidden p-0" initialFocus={false}>
            <TierDetailsDialog tier={plainTier} color="#d8b54a" />
        </DialogPopup>
    </Dialog>
);

export const EmptyTier = () => (
    <Dialog open>
        <DialogPopup className="max-w-2xl overflow-hidden p-0" initialFocus={false}>
            <TierDetailsDialog tier={emptyTier} color="#52b9b3" />
        </DialogPopup>
    </Dialog>
);
