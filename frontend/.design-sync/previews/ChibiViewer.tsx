import { ChibiViewer } from "frontend";

// The battle/dorm chibi player from the Skins tab (ported from SkinsContent.tsx,
// which passes the `/api/chibis/<id>` entry straight through). It mounts a PixiJS
// canvas and streams the Spine skeleton from the live asset host, so the card
// shows the real toolbar: view select, animation select and the export button.
//
// Fixture is the verbatim `/api/chibis/char_4064_mlynar` payload for the default
// and "epoque#28" skins.
const MLYNAR_DEFAULT_SKIN = {
    name: "default",
    path: "/spine/BattleFront/char_4064_mlynar",
    hasSpineData: true,
    animationTypes: {
        front: {
            atlas: "/spine/BattleFront/char_4064_mlynar/char_4064_mlynar.atlas",
            skel: "/spine/BattleFront/char_4064_mlynar/char_4064_mlynar.skel",
            png: "/spine/BattleFront/char_4064_mlynar/char_4064_mlynar.png",
        },
        back: {
            atlas: "/spine/BattleBack/char_4064_mlynar/char_4064_mlynar.atlas",
            skel: "/spine/BattleBack/char_4064_mlynar/char_4064_mlynar.skel",
            png: "/spine/BattleBack/char_4064_mlynar/char_4064_mlynar.png",
        },
        dorm: {
            atlas: "/spine/Building/char_4064_mlynar/build_char_4064_mlynar.atlas",
            skel: "/spine/Building/char_4064_mlynar/build_char_4064_mlynar.skel",
            png: "/spine/Building/char_4064_mlynar/build_char_4064_mlynar.png",
        },
    },
};

// A skin whose spine set was never shipped - the viewer falls back to its
// "No spine data available" message.
const MLYNAR_MISSING_SKIN = {
    name: "iteration#3",
    path: "/spine/BattleFront/char_4064_mlynar",
    hasSpineData: false,
    animationTypes: { front: { atlas: null, skel: null, png: null } },
};

const MLYNAR = {
    operatorCode: "char_4064_mlynar",
    name: "char_4064_mlynar",
    path: "char_4064_mlynar",
    skins: [MLYNAR_DEFAULT_SKIN, MLYNAR_MISSING_SKIN],
};

export const BattleChibi = () => (
    <div className="max-w-md">
        <ChibiViewer chibi={MLYNAR} server="en" skin={MLYNAR_DEFAULT_SKIN} />
    </div>
);

export const MissingSpineData = () => (
    <div className="max-w-md">
        <ChibiViewer chibi={MLYNAR} server="en" skin={MLYNAR_MISSING_SKIN} />
    </div>
);

export const AwaitingCatalog = () => (
    <div className="max-w-md">
        <ChibiViewer chibi={null} server="en" skin={null} />
    </div>
);
