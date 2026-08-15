import { DynamicArtOverlay } from "frontend";

// The overlay renders nothing unless a DynamicArtProvider has resolved a complete
// dyn_illust spine set for the operator's equipped skin. In the design bundle the
// chibi catalog is fetched through a stubbed server function, so these stories show
// the honest resting state: the host surface with its static artwork intact and the
// overlay layered on top, click-through and inactive.

const TILES = [
    { code: "char_4064_mlynar", name: "Mlynar", skin: "char_4064_mlynar#2" },
    { code: "char_263_skadi", name: "Skadi", skin: "char_263_skadi#2" },
    { code: "char_180_amgoat", name: "Eyjafjalla", skin: "char_180_amgoat#2" },
    { code: "char_1028_texas2", name: "Texas the Omertosa", skin: "char_1028_texas2#2" },
];

export const InactiveOverStaticArt = () => (
    <div className="relative h-96 w-72 overflow-hidden rounded-2xl border border-border bg-card">
        <img alt="Mlynar" className="h-full w-full object-cover" src="https://api.myrtle.moe/api/portrait/char_4064_mlynar" />
        <DynamicArtOverlay operatorCode="char_4064_mlynar" skinId="char_4064_mlynar#2" />
        <div className="absolute inset-x-0 bottom-0 flex items-end justify-between gap-2 bg-linear-to-t from-black/70 to-transparent px-4 pt-10 pb-3">
            <span className="font-sans font-semibold text-[15px] text-white leading-tight tracking-tight">Mlynar</span>
            <span className="rounded-full border border-white/25 bg-black/40 px-2 py-0.5 font-medium font-mono text-[10px] text-white uppercase leading-none tracking-[0.14em]">E2 Art</span>
        </div>
    </div>
);

export const ViewportGatedRosterTiles = () => (
    <div className="grid w-full max-w-2xl grid-cols-4 gap-3">
        {TILES.map((tile) => (
            <div className="relative aspect-3/4 overflow-hidden rounded-xl border border-border bg-muted" key={tile.code}>
                <img alt={tile.name} className="h-full w-full object-cover" src={`https://api.myrtle.moe/api/portrait/${tile.code}`} />
                <DynamicArtOverlay operatorCode={tile.code} skinId={tile.skin} viewportGated />
                <span className="absolute inset-x-0 bottom-0 truncate bg-linear-to-t from-black/70 to-transparent px-2 pt-6 pb-1.5 font-sans font-semibold text-[11px] text-white leading-tight">{tile.name}</span>
            </div>
        ))}
    </div>
);
