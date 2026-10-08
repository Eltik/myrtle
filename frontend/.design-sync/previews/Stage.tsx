import { Stage } from "frontend";
import type { ComponentProps } from "react";

// The frame the scene is drawn into: the 1280x720 canvas box centred in the
// stage, the background cover-scaled into it, character slots, the blocker
// tint over everything, and (by default) the background redrawn blurred and
// dimmed across any stage area outside the box. Frames here are hand-built the
// way the stage tests build them, from real 15-8 assets (Decompression Syndrome).

type Frame = NonNullable<ComponentProps<typeof Stage>["frame"]>;
type SceneState = Frame["state"];
type Slot = SceneState["slots"][keyof SceneState["slots"]];

const bg = (name: string, url: string) => ({ name, url, x: 0, y: 0, xScale: 1, yScale: 1, adapt: "coverall" as const, widthMul: 1, heightMul: 1, rotate: 0 });
const INFIRMARY = bg("bg_infirmary", "/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_infirmary.png");
const CEO = bg("bg_ceo", "/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png");

const AMIYA = {
    bodyUrl: "/textures/avg/characters/avg_1037_amiya3_1/avg_1037_amiya3_1$2.png",
    faceUrl: "/textures/avg/characters/avg_1037_amiya3_1/1$2.png",
    facePos: { x: 570, y: 233, w: 117, h: 95 },
    bodySize: { w: 1280, h: 1280 },
    plate: { x: 0, y: 155, w: 1070, h: 1070 },
};
const KALTSIT = {
    bodyUrl: "/textures/avg/characters/avg_003_kalts_1/avg_003_kalts_1$1.png",
    faceUrl: "/textures/avg/characters/avg_003_kalts_1/1$1.png",
    facePos: { x: 551, y: 62, w: 136, h: 181 },
    bodySize: { w: 1280, h: 1280 },
    plate: { x: 0, y: 180, w: 890, h: 890 },
};

function slot(sprite: object, name: string, lit: boolean): Slot {
    return { sprite, name, lit, x: 0, y: 0, alpha: 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap: 1 } as unknown as Slot;
}

function frame(over: Partial<SceneState>): Frame {
    const state = { blocker: { a: 0, r: 0, g: 0, b: 0 }, slots: {}, dialogVisible: false, stickers: {}, effects: {}, focus: {}, interludes: {}, ...over } as SceneState;
    return { state, transitionSec: 0, holdSec: 0, blocking: true } as Frame;
}

// Two speakers: Kal'tsit lit on the left, Amiya on the right dimmed while she
// listens. The lit slot draws last, so it sits in front.
export const TwoSpeakers = () => <Stage label="Story stage" frame={frame({ background: INFIRMARY, slots: { l: slot(KALTSIT, "kalts", true), r: slot(AMIYA, "amiya3", false) } })} />;

// One speaker centred, the commonest frame in the corpus.
export const SingleSpeaker = () => <Stage label="Story stage" frame={frame({ background: INFIRMARY, slots: { m: slot(AMIYA, "amiya3", true) } })} />;

// Background only, mid fade-to-black: the blocker at 60% black over the scene.
export const BlockerFade = () => <Stage label="Story stage" frame={frame({ background: CEO, blocker: { a: 0.6, r: 0, g: 0, b: 0 } })} />;

// A stage wider than 16:9 under the default "extend" trade: the background is
// continued, blurred and dimmed, either side of the canvas box.
export const ExtendFill = () => <Stage label="Story stage" className="h-80" frame={frame({ background: CEO, slots: { m: slot(KALTSIT, "kalts", true) } })} />;

// The same stage with "Letterbox like the game": black outside the box.
export const LetterboxMask = () => <Stage label="Story stage" className="h-80" fill="mask" frame={frame({ background: CEO, slots: { m: slot(KALTSIT, "kalts", true) } })} />;

// No frame yet (the script is still loading): the empty black stage.
export const Empty = () => <Stage label="Story stage" frame={null as unknown as Frame} />;
