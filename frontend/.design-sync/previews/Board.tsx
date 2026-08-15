import { Board } from "frontend";

/** Raw tile records exactly as `GET /level/{stageId}` ships them. */
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

/** 4-10 — Extinguished Flames: 11x7, three lanes, two teleport pairs. Sketched top row first. */
const SKETCH = [
    "...........", //
    "Srrfrrrfri.",
    ".ww.www.ww.",
    "SirfrrrrroE",
    ".wwwwwwwww.",
    "SorfrrrfrrE",
    "...........",
];
const WIDTH = SKETCH[0].length;
const HEIGHT = SKETCH.length;

/** `tiles` is row-major from the BOTTOM row up, which is the order the level file uses. */
const LEVEL = {
    mapData: {
        map: SKETCH.map((_, r) => Array.from({ length: WIDTH }, (_, c) => (HEIGHT - 1 - r) * WIDTH + c)),
        tiles: [...SKETCH].reverse().flatMap((row) => [...row].map((ch) => T[ch as keyof typeof T])),
    },
    routes: [],
    waves: [],
    tilesDisallowToLocate: [],
};

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;
const NO_OPERATORS = new Map();
/** A predefined squad, keyed by `row * width + col`. */
const SQUAD = new Map<number, { is_token: boolean; direction: number; char_key: string; icon: string }>([
    [3 * WIDTH + 3, { is_token: false, direction: 1, char_key: "char_4064_mlynar", icon: AVATAR("char_4064_mlynar") }],
    [5 * WIDTH + 4, { is_token: false, direction: 1, char_key: "char_263_skadi", icon: AVATAR("char_263_skadi") }],
    [1 * WIDTH + 7, { is_token: false, direction: 3, char_key: "char_102_texas", icon: AVATAR("char_102_texas") }],
]);

/** The board sits on MapView's dark, gridded surface at its natural 548x444. */
const Surface = ({ children }: { children?: React.ReactNode }) => (
    <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px]">
        <div className="relative mx-auto" style={{ width: 548, height: 444 }}>
            <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: 548, height: 444 }}>
                {children}
            </div>
        </div>
    </div>
);

/** The default 3D view: `hovered` tilts the grid 30° and extrudes high ground. */
export const ThreeDBoard = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered coordOverride={null} />
    </Surface>
);

/** Toggling 3D off flattens the board to a top-down plan. */
export const FlatBoard = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered={false} coordOverride={null} />
    </Surface>
);

/** `predefines.tokenInsts` paint deployed operators onto their tiles. */
export const WithPredefinedSquad = () => (
    <Surface>
        <Board level={LEVEL} operators={SQUAD} hovered coordOverride={null} />
    </Surface>
);

/** `tilesDisallowToLocate` strips the deployable outline from otherwise buildable tiles. */
export const RestrictedDeployment = () => (
    <Surface>
        <Board level={{ ...LEVEL, tilesDisallowToLocate: [3 * WIDTH + 3, 3 * WIDTH + 4, 3 * WIDTH + 5, 1 * WIDTH + 3, 1 * WIDTH + 7, 5 * WIDTH + 3, 5 * WIDTH + 7] }} operators={NO_OPERATORS} hovered coordOverride={null} />
    </Surface>
);

/** `coordOverride="maa"` swaps the A1-style labels for MAA's zero-indexed `col,row`. */
export const MaaCoordinates = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered coordOverride="maa" />
    </Surface>
);
