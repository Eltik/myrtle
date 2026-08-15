import { OperatorCardGrid } from "frontend";

// Rows lifted verbatim from GET https://api.myrtle.moe/api/operators/index and
// enriched the way `enrichOperators` does (voiceActors / hasNotes / ownership).
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, nationId: string, groupId: string | null, portraitSuffix: string, ownership: { owners: number; pct: number } | null) => ({
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
    teamId: null,
    artists: [],
    portrait: `/portraits/${id}${portraitSuffix}.png`,
    gender: "Female",
    race: "",
    placeOfBirth: "",
    voiceActors: [],
    hasNotes: false,
    ownership,
});

const SIX_STARS = [
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "kazimierz", null, "_2", { owners: 2914, pct: 0.3464 }),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "egir", "abyssal", "_2", { owners: 4408, pct: 0.5239 }),
    op("char_136_hsguma", "Hoshiguma", 6, "TANK", "protector", "lungmen", "lgd", "_2", { owners: 5230, pct: 0.6216 }),
    op("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "columbia", "rhine", "_2", { owners: 2183, pct: 0.2595 }),
    op("char_2025_shu", "Shu", 6, "TANK", "guardian", "yan", "sui", "_2", { owners: 1642, pct: 0.1952 }),
];

const RARITY_LADDER = [
    op("char_172_svrash", "SilverAsh", 6, "WARRIOR", "lord", "kjerag", "karlan", "_2", null),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "lungmen", "penguin", "_2", null),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "rhodes", null, "_2", null),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "rhodes", null, "_1", null),
    op("char_501_durin", "Durin", 2, "CASTER", "corecaster", "rhodes", null, "_1", null),
];

export const PortraitGrid = () => (
    <div className="grid w-full grid-cols-5 gap-3">
        {SIX_STARS.map((o) => (
            <OperatorCardGrid key={o.id} operator={o} />
        ))}
    </div>
);

export const RarityLadder = () => (
    <div className="grid w-full grid-cols-5 gap-3">
        {RARITY_LADDER.map((o) => (
            <OperatorCardGrid key={o.id} operator={o} />
        ))}
    </div>
);

export const SingleCard = () => (
    <div className="grid w-40 grid-cols-1">
        <OperatorCardGrid operator={SIX_STARS[0]} />
    </div>
);

const MORE = [
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "lungmen", "penguin", "_2", { owners: 5811, pct: 0.6907 }),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "columbia", "rhine", "_2", { owners: 7215, pct: 0.8577 }),
];

export const DenseGrid = () => (
    <div className="grid w-full grid-cols-6 gap-2.5">
        {[...SIX_STARS, ...MORE, ...RARITY_LADDER].map((o) => (
            <OperatorCardGrid key={o.id} operator={o} />
        ))}
    </div>
);
