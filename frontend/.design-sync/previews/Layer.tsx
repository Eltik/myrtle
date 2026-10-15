import { Layer } from "frontend";
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

// One image layer of the scene: the background, or a CG/image above the
// sprites. `coverall` scales the plate to cover the canvas, `showall` fits the plate
// inside it; `focus` is the focus-out blur the script applies.
// Real 15-8 (Decompression Syndrome) assets.

// A background, cover-scaled into the canvas.
export const Background = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
    </Canvas>
);

// A full-canvas CG image over a background (the 60_i06 plate, 1600x900 at 100 ppu).
export const CgImage = () => (
    <Canvas>
        <Layer layer={CORRIDOR} sec={0} kind="background" />
        <Layer layer={bg("60_i06", "/textures/avg/imgs/asp_60_0/60_i06.png", { adapt: "showall" })} sec={0} kind="image" />
    </Canvas>
);

// A background under the script's focus-out blur.
export const FocusedOut = () => (
    <Canvas>
        <Layer layer={INFIRMARY} sec={0} kind="background" focus={1} />
    </Canvas>
);
