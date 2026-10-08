import { InterludePanels, Layer } from "frontend";
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

// `interlude`: the radio-call windows. Each channel is a masked window over
// the scene, at canvas pixels from the centre, holding a background and/or a
// character, with an optional name tag. Mask sizes are the prefab's own
// (`interludeMask`): the vertical common window 290x760 (254x724 content), the
// square character window 300 root / 250 frame / 214 content.

const vertical = { maskId: "group_interclude_vertical_common", w: 290, h: 760, frameW: 290, frameH: 760, contentW: 254, contentH: 724, scaleX: 1, scaleY: 1, speaking: true };
const square = { maskId: "group_interclude_char", w: 300, h: 300, frameW: 250, frameH: 250, contentW: 214, contentH: 214, scaleX: 1, scaleY: 1, speaking: true };
const el = { x: 0, y: 0, scaleX: 1, scaleY: 1, alpha: 1 };

// Two vertical windows, a character in each, over the office.
export const TwoChannels = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <InterludePanels
            sec={0}
            panels={{
                a: { ...vertical, x: -250, y: 0, bg: { url: INFIRMARY.url, name: INFIRMARY.name }, character: { ...el, x: -250, name: "kalts", sprite: KALTSIT } },
                b: { ...vertical, x: 250, y: 0, bg: { url: CORRIDOR.url, name: CORRIDOR.name }, character: { ...el, x: 250, name: "amiya3", sprite: AMIYA } },
            } as never}
        />
    </Canvas>
);

// A square character window with its name tag.
export const NamedSquare = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" />
        <InterludePanels sec={0} panels={{ a: { ...square, x: -200, y: 170, label: "Kal'tsit", character: { ...el, x: -200, y: 40, name: "kalts", sprite: KALTSIT } } } as never} />
    </Canvas>
);

// A window holding a background only, the moment before a speaker appears.
export const BackgroundOnly = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <InterludePanels sec={0} panels={{ a: { ...vertical, x: 0, y: 0, speaking: false, bg: { url: CORRIDOR.url, name: CORRIDOR.name } } } as never} />
    </Canvas>
);
