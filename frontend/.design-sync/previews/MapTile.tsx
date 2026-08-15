import { MapTile } from "frontend";

/** Raw tile records exactly as `GET /level/{stageId}` ships them. */
const TILE = {
    forbidden: { tileKey: "tile_forbidden", heightType: "HIGHLAND", buildableType: "NONE", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    start: { tileKey: "tile_start", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    end: { tileKey: "tile_end", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    road: { tileKey: "tile_road", heightType: "LOWLAND", buildableType: "MELEE", passableMask: "ALL", playerSideMask: "ALL" },
    floor: { tileKey: "tile_floor", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    wall: { tileKey: "tile_wall", heightType: "HIGHLAND", buildableType: "RANGED", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    telin: { tileKey: "tile_telin", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    telout: { tileKey: "tile_telout", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    flystart: { tileKey: "tile_flystart", heightType: "HIGHLAND", buildableType: "NONE", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    hole: { tileKey: "tile_hole", heightType: "LOWLAND", buildableType: "NONE", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    healing: { tileKey: "tile_healing", heightType: "LOWLAND", buildableType: "MELEE", passableMask: "ALL", playerSideMask: "ALL" },
    defup: { tileKey: "tile_defup", heightType: "LOWLAND", buildableType: "MELEE", passableMask: "ALL", playerSideMask: "ALL" },
    gazebo: { tileKey: "tile_gazebo", heightType: "HIGHLAND", buildableType: "RANGED", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
    grass: { tileKey: "tile_grass", heightType: "LOWLAND", buildableType: "MELEE", passableMask: "ALL", playerSideMask: "ALL" },
    fence: { tileKey: "tile_fence", heightType: "LOWLAND", buildableType: "NONE", passableMask: "ALL", playerSideMask: "ALL" },
    deepwater: { tileKey: "tile_deepwater", heightType: "LOWLAND", buildableType: "NONE", passableMask: "FLY_ONLY", playerSideMask: "ALL" },
};

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;
const LABEL_STYLE = { color: "hsla(0,0%,78%,0.92)" };

const ctx = (over: Record<string, unknown> = {}) => ({ width: 11, height: 7, coordOverride: null, operators: new Map(), disallow: new Set<number>(), ...over });

/** The map's dark surface plus the perspective rig MapView wraps the board in. */
const Surface = ({ children }: { children?: React.ReactNode }) => (
    <div className="relative w-fit overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px] px-8 py-8">
        <div className="perspective-[900px] perspective-origin-[50%] transform-3d">{children}</div>
    </div>
);

const Labelled = ({ label, children }: { label: string; children?: React.ReactNode }) => (
    <div className="flex flex-col items-center gap-2">
        {children}
        <span className="font-medium font-mono text-[9px] uppercase tracking-[0.1em]" style={LABEL_STYLE}>
            {label}
        </span>
    </div>
);

/** 4-10's whole tile vocabulary, flat — the 2D view. */
export const BoardVocabulary = () => (
    <Surface>
        <div className="transform-3d flex flex-wrap items-center gap-6">
            <Labelled label="Start">
                <MapTile def={TILE.start} row={3} col={0} ctx={ctx()} />
            </Labelled>
            <Labelled label="End">
                <MapTile def={TILE.end} row={3} col={10} ctx={ctx()} />
            </Labelled>
            <Labelled label="Road">
                <MapTile def={TILE.road} row={3} col={2} ctx={ctx()} />
            </Labelled>
            <Labelled label="Floor">
                <MapTile def={TILE.floor} row={3} col={3} ctx={ctx()} />
            </Labelled>
            <Labelled label="High Ground">
                <MapTile def={TILE.wall} row={2} col={1} ctx={ctx()} />
            </Labelled>
            <Labelled label="Forbidden">
                <MapTile def={TILE.forbidden} row={0} col={0} ctx={ctx()} />
            </Labelled>
            <Labelled label="Teleport In">
                <MapTile def={TILE.telin} row={3} col={1} ctx={ctx()} />
            </Labelled>
            <Labelled label="Teleport Out">
                <MapTile def={TILE.telout} row={3} col={9} ctx={ctx()} />
            </Labelled>
        </div>
    </Surface>
);

/** `hovered` is the 3D view: boxes extrude, coordinates scale up, content lifts toward the camera. */
export const TiltedThreeD = () => (
    <Surface>
        <div className="map-hovered transform-3d flex flex-col-reverse items-center justify-center gap-y-0.5 rotate-x-30">
            {[
                [TILE.start, TILE.road, TILE.floor, TILE.road, TILE.end],
                [TILE.forbidden, TILE.wall, TILE.wall, TILE.wall, TILE.forbidden],
                [TILE.flystart, TILE.road, TILE.fence, TILE.hole, TILE.end],
            ].map((row, r) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: fixed 3x5 sample grid
                <div key={`row-${r}`} className="transform-3d flex flex-row items-center justify-start gap-x-0.5">
                    {row.map((def, c) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: fixed 3x5 sample grid
                        <MapTile key={`tile-${r}-${c}`} def={def} row={r} col={c} ctx={ctx({ hovered: true, width: 5, height: 3 })} />
                    ))}
                </div>
            ))}
        </div>
    </Surface>
);

/** Effect tiles carry an overlay glyph and an animated CSS effect layer. */
export const EffectTiles = () => (
    <Surface>
        <div className="transform-3d flex flex-wrap items-center gap-6">
            <Labelled label="Healing">
                <MapTile def={TILE.healing} row={2} col={4} ctx={ctx()} />
            </Labelled>
            <Labelled label="DEF Up">
                <MapTile def={TILE.defup} row={2} col={5} ctx={ctx()} />
            </Labelled>
            <Labelled label="Gazebo">
                <MapTile def={TILE.gazebo} row={2} col={6} ctx={ctx()} />
            </Labelled>
            <Labelled label="Grass">
                <MapTile def={TILE.grass} row={2} col={7} ctx={ctx()} />
            </Labelled>
            <Labelled label="Deep Water">
                <MapTile def={TILE.deepwater} row={2} col={8} ctx={ctx()} />
            </Labelled>
        </div>
    </Surface>
);

/** A tile in `tilesDisallowToLocate` loses its deployable outline; an occupied one shows its Marker. */
export const DeployableStates = () => (
    <Surface>
        <div className="transform-3d flex flex-wrap items-center gap-6">
            <Labelled label="Deployable">
                <MapTile def={TILE.road} row={3} col={2} ctx={ctx()} />
            </Labelled>
            <Labelled label="Disallowed">
                <MapTile def={TILE.road} row={3} col={2} ctx={ctx({ disallow: new Set([3 * 11 + 2]) })} />
            </Labelled>
            <Labelled label="Occupied">
                <MapTile def={TILE.road} row={3} col={4} ctx={ctx({ operators: new Map([[3 * 11 + 4, { is_token: false, direction: 1, char_key: "char_4064_mlynar", icon: AVATAR("char_4064_mlynar") }]]) })} />
            </Labelled>
            <Labelled label="MAA Coords">
                <MapTile def={TILE.road} row={3} col={4} ctx={ctx({ coordOverride: "maa" })} />
            </Labelled>
        </div>
    </Surface>
);
