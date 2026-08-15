import { OperatorCardUpcoming } from "frontend";

// Rows lifted verbatim from GET https://api.myrtle.moe/api/upcoming — CN
// operators that have not reached Global yet, so the names are still Chinese
// and the art is read from the `cn` asset server.
const up = (id: string, name: string, appellation: string, rarity: number, profession: string, subProfessionId: string, nationId: string) => ({
    id,
    name,
    appellation,
    rarity,
    profession,
    subProfessionId,
    position: "RANGED",
    tagList: [],
    nationId,
    isNotObtainable: false,
    groupId: null,
    teamId: null,
    artists: [],
    portrait: `/portraits/${id}_2.png`,
    gender: "Female",
    race: "",
    placeOfBirth: "",
});

const SIX_STARS = [
    up("char_1015_aglna2", "予愿安洁莉娜", "Angelina the Mellow Wish", 6, "SPECIAL", "skywalker", "rhodes"),
    up("char_1052_kalts2", "凯尔希·思衡托", "Kal'tsit·Esperanta", 6, "MEDIC", "watchman", "rhodes"),
    up("char_4228_closur", "可露希尔", "Closure", 6, "PIONEER", "tactician", "rhodes"),
    up("char_1051_headb2", "怒潮凛冬", "Zima the Raging Tide", 6, "WARRIOR", "hammer", "ursus"),
    up("char_4230_mcnist", "机械师", "Mechanist", 6, "TANK", "shotprotector", "rhodes"),
];

const MIXED = [
    up("char_4226_veen", "维伊", "Вий", 6, "CASTER", "mystic", "ursus"),
    up("char_4229_aphris", "谬因", "Aphrissa", 6, "CASTER", "blastcaster", "columbia"),
    up("char_4235_thumpy", "珊比", "Thumpy", 6, "TANK", "primprotector", "rim"),
    up("char_4224_turdus", "乌啾", "Укусик", 5, "MEDIC", "chainhealer", "ursus"),
    up("char_4236_tmslot", "时隙", "Timeslot", 5, "CASTER", "funnel", "rim"),
    up("char_4227_gallus", "GALLUS²", "GALLUS²", 1, "CASTER", "corecaster", "rhodes"),
];

export const UpcomingSixStars = () => (
    <div className="grid w-full grid-cols-5 gap-3">
        {SIX_STARS.map((o) => (
            <OperatorCardUpcoming key={o.id} operator={o} />
        ))}
    </div>
);

export const MixedRarities = () => (
    <div className="grid w-full grid-cols-6 gap-2.5">
        {MIXED.map((o) => (
            <OperatorCardUpcoming key={o.id} operator={o} />
        ))}
    </div>
);

export const SingleCard = () => (
    <div className="grid w-40 grid-cols-1">
        <OperatorCardUpcoming operator={SIX_STARS[0]} />
    </div>
);
