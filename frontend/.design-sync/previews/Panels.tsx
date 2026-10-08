import { Panels } from "frontend";
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

// `largebg` / `gridbg`: a background drawn out of several panels butted left
// to right (two rows for a gridbg), the whole strip panned by `x` in canvas
// pixels. Panels here are 15-8's own backgrounds standing in for a strip.

const P1 = "/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png";
const P2 = "/textures/avg/bg/avg_bkg_h1_60_0/60_g1_rhodescorridor_bc.png";

// A two-panel strip at its start: the first panel and the head of the second.
export const StripStart = () => (
    <Canvas>
        <Panels panels={{ urls: [P1, P2], widths: [1280, 1280], height: 720, rows: 1, x: 0, y: 0 }} sec={0} />
    </Canvas>
);

// The same strip panned 640 px left, straddling the seam.
export const StripPanned = () => (
    <Canvas>
        <Panels panels={{ urls: [P1, P2], widths: [1280, 1280], height: 720, rows: 1, x: -640, y: 0 }} sec={0} />
    </Canvas>
);
