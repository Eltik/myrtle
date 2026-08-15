import { OperatorHero } from "frontend";

// The banner at the top of `/operators/$id` (ported from
// src/components/operators/detail/Operators.tsx, which hands it the raw
// `/api/operators/<id>` payload). Rarity is the axis that actually changes the
// render: it drives the star row colour and the drop-shadow glow behind the
// splash art, so the three stories sweep 6* / 5* / 3*.
//
// Ids, art paths and faction ids are taken verbatim from the live API.

export const SixStarLibrator = () => (
    <OperatorHero
        operator={{
            id: "char_4064_mlynar",
            name: "Młynar",
            rarity: "TIER_6",
            profession: "WARRIOR",
            subProfessionId: "librator",
            position: "MELEE",
            nationId: "kazimierz",
            teamId: null,
            groupId: null,
            skin: "/textures/chararts/char_4064_mlynar/char_4064_mlynar_2.png",
            portrait: "/portraits/char_4064_mlynar_2.png",
            server: "en",
        }}
    />
);

export const FiveStarVanguard = () => (
    <OperatorHero
        operator={{
            id: "char_102_texas",
            name: "Texas",
            rarity: "TIER_5",
            profession: "PIONEER",
            subProfessionId: "pioneer",
            position: "MELEE",
            nationId: "lungmen",
            teamId: null,
            groupId: "penguin",
            skin: "/textures/chararts/char_102_texas/char_102_texas_2.png",
            portrait: "/portraits/char_102_texas_2.png",
            server: "en",
        }}
    />
);

export const ThreeStarGuard = () => (
    <OperatorHero
        operator={{
            id: "char_208_melan",
            name: "Melantha",
            rarity: "TIER_3",
            profession: "WARRIOR",
            subProfessionId: "fearless",
            position: "MELEE",
            nationId: "rhodes",
            teamId: "reserve4",
            groupId: null,
            skin: "/textures/chararts/char_208_melan/char_208_melan_1.png",
            portrait: "/portraits/char_208_melan_1.png",
            server: "en",
        }}
    />
);
