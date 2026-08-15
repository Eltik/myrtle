import { SceneIllust } from "frontend";

// The Live2D scene renderer behind the Skins tab. It fills its positioned
// parent (`absolute inset-0`), streams the DynIllust Spine set from the asset
// host and draws the static skin art underneath as the backdrop, so its preview
// is the skin frame it lives in - ported from SkinsContent.tsx.
//
// Its own chrome is the bottom-right status pill: "Loading animation" while the
// skeleton streams, or the failure message when a set is missing.
const ASSETS = "https://api.myrtle.moe/api/assets";

const MLYNAR_EPOQUE = {
    atlas: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.atlas",
    skel: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.skel",
    png: "/spine/DynIllust/char_4064_mlynar_epoque#28/dyn_illust_char_4064_mlynar_epoque#28.png",
};
const MLYNAR_EPOQUE_ART = `${ASSETS}/textures/skinpack/char_4064_mlynar/char_4064_mlynar_epoque%2328.png`;

const TEXAS2_ITERATION = {
    atlas: "/spine/DynIllust/char_1028_texas2_iteration#1/dyn_illust_char_1028_texas2_iteration#1.atlas",
    skel: "/spine/DynIllust/char_1028_texas2_iteration#1/dyn_illust_char_1028_texas2_iteration#1.skel",
    png: "/spine/DynIllust/char_1028_texas2_iteration#1/dyn_illust_char_1028_texas2_iteration#1.png",
};
const TEXAS2_ITERATION_ART = `${ASSETS}/textures/skinpack/char_1028_texas2/char_1028_texas2_iteration%231.png`;

function SkinFrame({ art, kicker, name, artist, children }: { art: string; kicker: string; name: string; artist: string; children: React.ReactNode }) {
    return (
        <div className="relative aspect-4/3 overflow-hidden rounded-xl border border-border bg-linear-to-b from-secondary/30 via-secondary/10 to-secondary/40 md:aspect-16/11">
            <img alt={name} className="absolute inset-0 h-full w-full object-contain" decoding="async" loading="eager" src={art} />
            {children}
            <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/70 via-black/40 to-transparent" />
            <div className="absolute inset-x-0 bottom-0 p-4 md:p-5">
                <div className="font-mono text-[10px] text-white/70 uppercase tracking-wider">{kicker}</div>
                <div className="mt-0.5 font-semibold text-2xl text-white drop-shadow-md">{name}</div>
                <div className="mt-0.5 text-white/70 text-xs">Artist · {artist}</div>
            </div>
        </div>
    );
}

export const AuthoredFraming = () => (
    <SkinFrame art={MLYNAR_EPOQUE_ART} artist="竜崎いち" kicker="EPOQUE/XXVIII" name="W Dali">
        <SceneIllust backdrop={MLYNAR_EPOQUE_ART} files={MLYNAR_EPOQUE} framing="authored" server="en" />
    </SkinFrame>
);

export const CharacterFraming = () => (
    <SkinFrame art={TEXAS2_ITERATION_ART} artist="幻象黑兔" kicker="Iteration Provident" name="Wingbreaker">
        <SceneIllust backdrop={TEXAS2_ITERATION_ART} files={TEXAS2_ITERATION} framing="character" server="en" />
    </SkinFrame>
);

// The fullscreen branch of SkinViewerDialog: no static art underneath, just the
// scene on the dialog's own dark stage.
export const FullscreenStage = () => (
    <div className="relative h-96 w-full overflow-hidden rounded-xl border border-border bg-black">
        <SceneIllust backdrop={MLYNAR_EPOQUE_ART} files={MLYNAR_EPOQUE} framing="authored" server="en" />
    </div>
);
