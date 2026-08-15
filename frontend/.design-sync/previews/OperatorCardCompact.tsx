import { OperatorCardCompact } from "frontend";

// Rows lifted verbatim from GET https://api.myrtle.moe/api/operators/index and
// enriched the way `enrichOperators` does (voiceActors / hasNotes / ownership).
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, nationId: string, groupId: string | null, teamId: string | null, ownership: { owners: number; pct: number } | null) => ({
    id,
    name,
    appellation: name,
    rarity,
    profession,
    subProfessionId,
    position: "MELEE",
    tagList: [],
    nationId,
    isNotObtainable: false,
    groupId,
    teamId,
    artists: [],
    portrait: null,
    gender: "Female",
    race: "",
    placeOfBirth: "",
    voiceActors: [],
    hasNotes: false,
    ownership,
});

const ROSTER = [
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "kazimierz", null, null, { owners: 2914, pct: 0.3464 }),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "egir", "abyssal", null, { owners: 4408, pct: 0.5239 }),
    op("char_136_hsguma", "Hoshiguma", 6, "TANK", "protector", "lungmen", "lgd", null, { owners: 5230, pct: 0.6216 }),
    op("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "columbia", "rhine", null, { owners: 2183, pct: 0.2595 }),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "lungmen", "penguin", null, { owners: 6120, pct: 0.7275 }),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "columbia", "rhine", null, { owners: 7215, pct: 0.8577 }),
];

const RARITY_LADDER = [
    op("char_2025_shu", "Shu", 6, "TANK", "guardian", "yan", "sui", null, null),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "lungmen", "penguin", null, null),
    op("char_002_amiya", "Amiya", 5, "CASTER", "corecaster", "rhodes", null, null, null),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "rhodes", null, null, null),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "rhodes", null, "reserve1", null),
    op("char_501_durin", "Durin", 2, "CASTER", "corecaster", "rhodes", null, "action4", null),
];

const NO_OWNERSHIP = [
    op("char_172_svrash", "SilverAsh", 6, "WARRIOR", "lord", "kjerag", "karlan", null, null),
    op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "siracusa", null, null, null),
    op("char_017_huang", "Blaze", 6, "WARRIOR", "centurion", "rhodes", "elite", null, null),
    op("char_181_flower", "Perfumer", 4, "MEDIC", "ringhealer", "rhodes", null, null, null),
    op("char_123_fang", "Fang", 3, "PIONEER", "pioneer", "rhodes", null, "reserve1", null),
    op("char_209_ardign", "Cardigan", 3, "TANK", "protector", "rhodes", null, "reserve4", null),
];

export const CompactGrid = () => (
    <div className="grid w-full grid-cols-6 gap-1">
        {ROSTER.map((o) => (
            <OperatorCardCompact key={o.id} operator={o} />
        ))}
    </div>
);

export const RarityLadder = () => (
    <div className="grid w-full grid-cols-6 gap-1">
        {RARITY_LADDER.map((o) => (
            <OperatorCardCompact key={o.id} operator={o} />
        ))}
    </div>
);

export const OwnershipUnavailable = () => (
    <div className="grid w-full grid-cols-6 gap-1">
        {NO_OWNERSHIP.map((o) => (
            <OperatorCardCompact key={o.id} operator={o} />
        ))}
    </div>
);

export const SingleTile = () => (
    <div className="grid w-32 grid-cols-1 gap-1">
        <OperatorCardCompact operator={ROSTER[0]} />
    </div>
);
