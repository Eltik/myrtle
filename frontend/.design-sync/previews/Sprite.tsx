import { Layer, Sprite } from "frontend";
import type { ComponentProps, CSSProperties, ReactNode } from "react";

// The reader's 16:9 canvas box, rebuilt as Stage draws it: a size container
// whose one canvas pixel is `--story-cpx`, and a 1280x720 box centred in it.
// Every stage layer is positioned in those canvas pixels, so it needs this box.
function Canvas({ children }: { children: ReactNode }) {
    return (
        <section className="relative aspect-video w-full overflow-hidden rounded-lg bg-black [container-type:size]" style={{ "--story-cpx": "min(0.078125cqw, 0.1388889cqh)" } as CSSProperties}>
            <div className="absolute overflow-hidden" style={{ left: "50%", top: "50%", translate: "-50% -50%", width: "calc(1280 * var(--story-cpx))", height: "calc(720 * var(--story-cpx))" }}>
                {children}
            </div>
        </section>
    );
}

const bg = (name: string, url: string, over: object = {}) => ({ name, url, x: 0, y: 0, xScale: 1, yScale: 1, adapt: "coverall" as const, widthMul: 1, heightMul: 1, rotate: 0, ...over });
const CEO = bg("bg_ceo", "/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png");
const INFIRMARY = bg("bg_infirmary", "/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_infirmary.png");
const CORRIDOR = bg("60_g1_rhodescorridor_bc", "/textures/avg/bg/avg_bkg_h1_60_0/60_g1_rhodescorridor_bc.png");

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
const ROSMON = {
    bodyUrl: "/textures/avg/characters/avg_391_rosmon_1/avg_391_rosmon_1$2.png",
    faceUrl: "/textures/avg/characters/avg_391_rosmon_1/1$2.png",
    facePos: { x: 496, y: 250, w: 144, h: 149 },
    bodySize: { w: 1024, h: 1024 },
    plate: { x: -100, y: 270, w: 1220, h: 1220 },
};

type SlotState = ComponentProps<typeof Sprite>["state"];
const s = (sprite: object, name: string, lit: boolean, over: object = {}) => ({ sprite, name, lit, x: 0, y: 0, alpha: 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap: 1, ...over }) as unknown as SlotState;

// One character slot on the stage: the body texture on its authored plate,
// the expression face composited at `facePos`, placed in the left, middle or
// right slot. A slot that is not speaking is dimmed. Real 15-8 sprites.

// The middle slot, lit (speaking).
export const MiddleLit = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" />
        <Sprite slot="m" state={s(AMIYA, "amiya3", true)} sec={0} />
    </Canvas>
);

// Left lit, right dimmed: the two-slot conversation layout.
export const LeftAndRight = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" />
        <Sprite slot="r" state={s(AMIYA, "amiya3", false)} sec={0} />
        <Sprite slot="l" state={s(KALTSIT, "kalts", true)} sec={0} />
    </Canvas>
);

// A wide plate (Rosmontis, 1220 canvas px) in the middle slot.
export const WidePlate = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <Sprite slot="m" state={s(ROSMON, "rosmon", true)} sec={0} />
    </Canvas>
);

// A slot at half alpha and scaled up, mid-tween.
export const ScaledFaded = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <Sprite slot="m" state={s(KALTSIT, "kalts", true, { alpha: 0.5, scale: 1.2 })} sec={0} />
    </Canvas>
);
