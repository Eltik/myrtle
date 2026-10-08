import { ToolbarHandle } from "frontend";

// The collapsed toolbar's handle: a 24 px frosted tab hanging from the top edge
// of the stage, centred between where the two pills were. It is `absolute`, so
// it sits on the reader's 16:9 stage over the current scene.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png";
const noop = () => {};

// Toolbar hidden ("Hide toolbar" or theater mode): only the handle remains.
export const OverScene = () => (
    <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">
        <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
        <ToolbarHandle onPointerEnter={noop} onPointerLeave={noop} onClick={noop} />
    </div>
);

// The same handle over a black transition frame, where the glass is the only
// edge the reader has to find it by.
export const OverBlack = () => (
    <div className="relative h-40 w-full overflow-hidden rounded-lg bg-black">
        <ToolbarHandle onPointerEnter={noop} onPointerLeave={noop} onClick={noop} />
    </div>
);
