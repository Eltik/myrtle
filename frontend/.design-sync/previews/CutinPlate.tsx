import { CutinPlate, Layer } from "frontend";
import type { CSSProperties, ReactNode } from "react";

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

// The `cutin` command: a character's body clipped into a rectangular window
// laid over the scene, at `x`/`y` canvas pixels from the centre. The body
// sits on its own authored plate inside the window.

// A full-width band: the window shows the plate from the knees up.
export const Strip = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <CutinPlate cutin={{ sprite: KALTSIT, name: "kalts", x: 0, y: 0, width: 1280, height: 620 } as never} sec={0} />
    </Canvas>
);

// A tall window off to the right.
export const Portrait = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" />
        <CutinPlate cutin={{ sprite: AMIYA, name: "amiya3", x: 320, y: 0, width: 420, height: 720 } as never} sec={0} />
    </Canvas>
);
