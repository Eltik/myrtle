import { Layer, OverlayText } from "frontend";
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

// Text drawn straight onto the scene: `subtitle` (medium weight) and
// `sticker` (the system-message register). Placed in canvas pixels from the
// top-left, with the same markup a line carries (`<color>`, `<i>`). Lines
// are 15-8's opening PRTS stickers.

// A sticker block, left-aligned at (180, 170), as 15-8 opens.
export const Sticker = () => (
    <Canvas>
        <OverlayText kind="sticker" overlay={{ text: "Administrator permission found.\n\nPreparing reboot process from hibernation. 3... 2... 1...\n\nPRTS reboot diagnosis complete—", x: 180, y: 170, alignment: "left", size: 24, width: 700 }} />
    </Canvas>
);

// A sticker with a `<color>` run, the red warning line.
export const ColoredSticker = () => (
    <Canvas>
        <OverlayText kind="sticker" overlay={{ text: "<color=#ff0000>Warning: Unauthorized 3rd-party plugin found. Restore default settings?</color>", x: 180, y: 300, alignment: "left", size: 24, width: 700 }} />
    </Canvas>
);

// A centred subtitle over a scene.
export const Subtitle = () => (
    <Canvas>
        <Layer layer={CEO} sec={0} kind="background" />
        <OverlayText kind="subtitle" overlay={{ text: "<i>Rhodes Island, the CEO's office.</i>", x: 240, y: 560, alignment: "center", size: 30, width: 800 }} />
    </Canvas>
);
