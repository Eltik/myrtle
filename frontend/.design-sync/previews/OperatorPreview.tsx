import { OperatorPreview } from "frontend";

// Rows lifted verbatim from GET https://api.myrtle.moe/api/operators/index.
const MLYNAR = {
    id: "char_4064_mlynar",
    name: "Młynar",
    appellation: "Młynar",
    rarity: 6,
    profession: "WARRIOR",
    subProfessionId: "librator",
    position: "MELEE",
    tagList: ["DPS", "Nuker"],
    nationId: "kazimierz",
    isNotObtainable: false,
    groupId: null,
    teamId: null,
    artists: ["竜崎いち"],
    portrait: "/portraits/char_4064_mlynar_2.png",
    gender: "Male",
    race: "Kuranta",
    placeOfBirth: "Kazimierz",
};

const KROOS = {
    id: "char_124_kroos",
    name: "Kroos",
    appellation: "Kroos",
    rarity: 3,
    profession: "SNIPER",
    subProfessionId: "fastshot",
    position: "RANGED",
    tagList: ["DPS"],
    nationId: "rhodes",
    isNotObtainable: false,
    groupId: null,
    teamId: "reserve1",
    artists: ["下野宏铭"],
    portrait: "/portraits/char_124_kroos_1.png",
    gender: "Female",
    race: "Cautus",
    placeOfBirth: "Rim Billiton",
};

const WISADEL = {
    id: "char_1035_wisdel",
    name: "Wiš'adel",
    appellation: "Wiš'adel",
    rarity: 6,
    profession: "SNIPER",
    subProfessionId: "bombarder",
    position: "RANGED",
    tagList: ["DPS", "Slow"],
    nationId: "",
    isNotObtainable: false,
    groupId: "babel",
    teamId: null,
    artists: ["Liduke", "板板"],
    portrait: "/portraits/char_1035_wisdel_2.png",
    gender: "Female",
    race: "Sarkaz",
    placeOfBirth: "Kazdel",
};

const MUELSYSE = {
    id: "char_249_mlyss",
    name: "Muelsyse",
    appellation: "Muelsyse",
    rarity: 6,
    profession: "PIONEER",
    subProfessionId: "tactician",
    position: "RANGED",
    tagList: ["DP-Recovery", "Summon"],
    nationId: "columbia",
    isNotObtainable: false,
    groupId: "rhine",
    teamId: null,
    artists: ["NoriZC"],
    portrait: "/portraits/char_249_mlyss_2.png",
    gender: "Female",
    race: "Elf",
    placeOfBirth: "Undisclosed",
};

/** The popup shell `OperatorCardGrid` renders this inside (`HoverCardContent`). */
const Popup = ({ children }: { children: React.ReactNode }) => <div className="w-max max-w-85 overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-md">{children}</div>;

export const SixStarGuard = () => (
    <Popup>
        <OperatorPreview operator={MLYNAR} />
    </Popup>
);

export const ThreeStarSniper = () => (
    <Popup>
        <OperatorPreview operator={KROOS} />
    </Popup>
);

export const FactionFallback = () => (
    <Popup>
        <OperatorPreview operator={WISADEL} />
    </Popup>
);

export const RarityAccents = () => (
    <div className="flex flex-wrap gap-3">
        {[MUELSYSE, KROOS].map((op) => (
            <Popup key={op.id}>
                <OperatorPreview operator={op} />
            </Popup>
        ))}
    </div>
);
