import { OperatorCardList } from "frontend";

// LIST_GRID_COLS from src/components/operators/list/impl/constants.ts — the row
// header has to line up with the columns the card itself lays out.
const LIST_GRID_COLS = "52px 1fr 96px 128px 160px 88px 32px";

// Rows lifted verbatim from GET https://api.myrtle.moe/api/operators/index and
// enriched the way `enrichOperators` does (voiceActors / hasNotes / ownership).
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, nationId: string, groupId: string | null, ownership: { owners: number; pct: number } | null) => ({
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
    portrait: null,
    gender: "Female",
    race: "",
    placeOfBirth: "",
    voiceActors: [],
    hasNotes: false,
    ownership,
});

const OWNED = [
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "kazimierz", null, { owners: 2914, pct: 0.3464 }),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "egir", "abyssal", { owners: 4408, pct: 0.5239 }),
    op("char_136_hsguma", "Hoshiguma", 6, "TANK", "protector", "lungmen", "lgd", { owners: 5230, pct: 0.6216 }),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "lungmen", "penguin", { owners: 6120, pct: 0.7275 }),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "rhodes", null, { owners: 7984, pct: 0.9491 }),
];

const UNRANKED = [
    op("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "columbia", "rhine", null),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "columbia", "rhine", null),
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "rhodes", null, null),
    op("char_501_durin", "Durin", 2, "CASTER", "corecaster", "rhodes", null, null),
];

const HeaderRow = () => (
    <div className="grid items-center gap-3 rounded-lg border border-transparent border-b-border/60 px-3 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
        <span />
        <span>Name</span>
        <span className="text-center">Rarity</span>
        <span className="text-center">Class</span>
        <span className="text-center">Archetype</span>
        <span className="text-center">Owned</span>
        <span />
    </div>
);

export const ResultsTable = () => (
    <div className="flex w-full flex-col gap-1 pt-1">
        <HeaderRow />
        <div className="flex flex-col gap-1">
            {OWNED.map((o) => (
                <OperatorCardList key={o.id} operator={o} />
            ))}
        </div>
    </div>
);

export const WithoutOwnershipData = () => (
    <div className="flex w-full flex-col gap-1 pt-1">
        <HeaderRow />
        <div className="flex flex-col gap-1">
            {UNRANKED.map((o) => (
                <OperatorCardList key={o.id} operator={o} />
            ))}
        </div>
    </div>
);

export const SingleRow = () => (
    <div className="w-full">
        <OperatorCardList operator={OWNED[0]} />
    </div>
);
