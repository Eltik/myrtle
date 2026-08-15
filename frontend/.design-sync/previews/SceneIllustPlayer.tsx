import { SceneIllustPlayer } from "frontend";

// SSR-safe wrapper for the Live2D scene renderer: `ClientOnly` + `Suspense`
// around the PixiJS chunk, so nothing is emitted on the server and the static
// skin art below it stays visible until the animation is ready. Ported from
// SkinsContent.tsx (card frame) and SkinViewerDialog (fullscreen frame).
const ASSETS = "https://api.myrtle.moe/api/assets";

const MLYNAR_EPOQUE = {
    atlas: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.atlas",
    skel: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.skel",
    png: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.png",
};
const MLYNAR_EPOQUE_ART = `${ASSETS}/textures/skinpack/char_4064_mlynar/char_4064_mlynar_epoque%2328.png`;

const TEXAS2_EPOQUE = {
    atlas: "/spine/DynIllust/char_1028_texas2_epoque#36/dyn_illust_char_1028_texas2_epoque#36.atlas",
    skel: "/spine/DynIllust/char_1028_texas2_epoque#36/dyn_illust_char_1028_texas2_epoque#36.skel",
    png: "/spine/DynIllust/char_1028_texas2_epoque#36/dyn_illust_char_1028_texas2_epoque#36.png",
};
const TEXAS2_EPOQUE_ART = `${ASSETS}/textures/skinpack/char_1028_texas2/char_1028_texas2_epoque%2336.png`;

export const SkinCard = () => (
    <div className="relative aspect-4/3 overflow-hidden rounded-xl border border-border bg-linear-to-b from-secondary/30 via-secondary/10 to-secondary/40 md:aspect-16/11">
        <img alt="W Dali" className="absolute inset-0 h-full w-full object-contain" decoding="async" loading="eager" src={MLYNAR_EPOQUE_ART} />
        <SceneIllustPlayer backdrop={MLYNAR_EPOQUE_ART} files={MLYNAR_EPOQUE} framing="authored" server="en" />
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/70 via-black/40 to-transparent" />
        <div className="absolute inset-x-0 bottom-0 p-4 md:p-5">
            <div className="font-mono text-[10px] text-white/70 uppercase tracking-wider">EPOQUE/XXVIII</div>
            <div className="mt-0.5 font-semibold text-2xl text-white drop-shadow-md">W Dali</div>
            <div className="mt-0.5 text-white/70 text-xs">Artist · 竜崎いち</div>
        </div>
    </div>
);

export const FullscreenViewer = () => (
    <div className="relative h-96 w-full overflow-hidden rounded-xl border border-border bg-black">
        <SceneIllustPlayer backdrop={TEXAS2_EPOQUE_ART} files={TEXAS2_EPOQUE} framing="authored" server="en" />
    </div>
);
