import { DynamicChibiViewer } from "frontend";

// SSR-safe wrapper around `ChibiViewer`: the PixiJS scene only exists on the
// client, so this renders a skeleton (matching the viewer's toolbar + stage
// footprint) until the lazy chunk resolves. Ported from SkinsContent.tsx, which
// only mounts it once the `/api/chibis/<id>` lookup has a character.
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

const SKADI_DORM_SKIN = {
    name: "default",
    path: "/spine/BattleFront/char_263_skadi",
    hasSpineData: true,
    animationTypes: {
        front: {
            atlas: "/spine/BattleFront/char_263_skadi/char_263_skadi.atlas",
            skel: "/spine/BattleFront/char_263_skadi/char_263_skadi.skel",
            png: "/spine/BattleFront/char_263_skadi/char_263_skadi.png",
        },
        back: {
            atlas: "/spine/BattleBack/char_263_skadi/char_263_skadi.atlas",
            skel: "/spine/BattleBack/char_263_skadi/char_263_skadi.skel",
            png: "/spine/BattleBack/char_263_skadi/char_263_skadi.png",
        },
        dorm: {
            atlas: "/spine/Building/char_263_skadi/build_char_263_skadi.atlas",
            skel: "/spine/Building/char_263_skadi/build_char_263_skadi.skel",
            png: "/spine/Building/char_263_skadi/build_char_263_skadi.png",
        },
    },
};

const character = (code: string, skin: unknown) => ({ operatorCode: code, name: code, path: code, skins: [skin] });

export const MlynarBattleChibi = () => (
    <div className="max-w-md">
        <DynamicChibiViewer chibi={character("char_4064_mlynar", MLYNAR_DEFAULT_SKIN)} server="en" skin={MLYNAR_DEFAULT_SKIN} />
    </div>
);

export const SkadiAllViews = () => (
    <div className="max-w-md">
        <DynamicChibiViewer chibi={character("char_263_skadi", SKADI_DORM_SKIN)} server="en" skin={SKADI_DORM_SKIN} />
    </div>
);
