import { Board, TileLegend } from "frontend";

const T = {
    ".": { tileKey: "tile_forbidden", heightType: "HIGHLAND", buildableType: "NONE", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    S: { tileKey: "tile_start", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    E: { tileKey: "tile_end", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    r: { tileKey: "tile_road", heightType: "LOWLAND", buildableType: "MELEE", passableMask: "ALL", playerSideMask: "ALL" },
    f: { tileKey: "tile_floor", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    w: { tileKey: "tile_wall", heightType: "HIGHLAND", buildableType: "RANGED", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    i: { tileKey: "tile_telin", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    o: { tileKey: "tile_telout", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
};
/** 4-10 — Extinguished Flames, sketched top row first. */
const SKETCH = ["...........", "Srrfrrrfri.", ".ww.www.ww.", "SirfrrrrroE", ".wwwwwwwww.", "SorfrrrfrrE", "..........."];
const WIDTH = SKETCH[0].length;
const HEIGHT = SKETCH.length;
const LEVEL = {
    mapData: {
        map: SKETCH.map((_, r) => Array.from({ length: WIDTH }, (_, c) => (HEIGHT - 1 - r) * WIDTH + c)),
        tiles: [...SKETCH].reverse().flatMap((row) => [...row].map((ch) => T[ch as keyof typeof T])),
    },
    tilesDisallowToLocate: [],
};

export const Standalone = () => (
    <div className="max-w-3xl">
        <TileLegend />
    </div>
);

/** Where it sits on the stage page: directly under the board, above the map settings. */
export const UnderTheBoard = () => (
    <div className="flex max-w-3xl flex-col gap-3">
        <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px]">
            <div className="relative mx-auto" style={{ width: 548, height: 444 }}>
                <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: 548, height: 444 }}>
                    <Board level={LEVEL} operators={new Map()} hovered coordOverride={null} />
                </div>
            </div>
        </div>
        <TileLegend />
    </div>
);

/** In a narrow column the swatch row wraps instead of clipping. */
export const Narrow = () => (
    <div className="max-w-xs">
        <TileLegend />
    </div>
);
