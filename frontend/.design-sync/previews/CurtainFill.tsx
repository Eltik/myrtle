import { CurtainFill, Layer } from "frontend";
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

// The `curtain` command: a black sheet closing over the scene from one edge,
// `fill` of the way across (direction 0/1/7 top, 2 right, 3-5 bottom, 6 left).
// Drawn over the background inside the canvas box.

// Closing from the top, half way, hard edge.
export const FromTop = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <CurtainFill curtain={{ direction: 0, fill: 0.5, grad: false }} sec={0} />
    </Canvas>
);

// Closing from the left, 40%.
export const FromLeft = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" />
        <CurtainFill curtain={{ direction: 6, fill: 0.4, grad: false }} sec={0} />
    </Canvas>
);

// Rising from the bottom, 30%.
export const FromBottom = () => (
    <Canvas>
        <Layer layer={CORRIDOR} sec={0} kind="background" />
        <CurtainFill curtain={{ direction: 4, fill: 0.3, grad: false }} sec={0} />
    </Canvas>
);
