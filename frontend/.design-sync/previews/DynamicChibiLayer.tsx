import { Board, DynamicChibiLayer, RoutesLayer } from "frontend";

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

const BOARD_W = 548;
const BOARD_H = 444;

/** Spine sets served by `GET /static/enemy-chibis` — the `_2` variants live in the base folder. */
const SPINE: Record<string, { skel: string; atlas: string }> = {
    enemy_1505_frstar: { skel: "/spine/Enemy/enemy_1505_frstar/enemy_1505_frstar.skel", atlas: "/spine/Enemy/enemy_1505_frstar/enemy_1505_frstar.atlas" },
    enemy_1032_katar: { skel: "/spine/Enemy/enemy_1032_katar/enemy_1032_katar.skel", atlas: "/spine/Enemy/enemy_1032_katar/enemy_1032_katar.atlas" },
    enemy_1029_shdsbr: { skel: "/spine/Enemy/enemy_1029_shdsbr/enemy_1029_shdsbr.skel", atlas: "/spine/Enemy/enemy_1029_shdsbr/enemy_1029_shdsbr.atlas" },
    enemy_1034_laxe: { skel: "/spine/Enemy/enemy_1034_laxe/enemy_1034_laxe.skel", atlas: "/spine/Enemy/enemy_1034_laxe/enemy_1034_laxe.atlas" },
};
const spine = (id: string) => SPINE[id];
const LANE_TOP = "M 24 74 L 124 74 L 324 74 L 474 74 L 474 174 L 424 174 L 174 174 L 74 174 L 74 274 L 174 274 L 524 274";
const LANE_MID = "M 24 174 L 524 174";
const LANE_BOTTOM = "M 24 274 L 524 274";

const WALKERS = [
    { position: 0, enemyKey: "enemy_1505_frstar", d: LANE_TOP, speed: 25, waits: [{ dist: 100, wait: 10 }], ...spine("enemy_1505_frstar") },
    { position: 7, enemyKey: "enemy_1034_laxe", d: LANE_MID, speed: 35, waits: [], ...spine("enemy_1034_laxe") },
    { position: 8, enemyKey: "enemy_1032_katar", d: LANE_BOTTOM, speed: 50, waits: [], ...spine("enemy_1032_katar") },
];

const ROUTE_ENTRIES = [
    {
        position: 0,
        timestamp: 3,
        enemyIndexRange: "1",
        enemyKey: "enemy_1505_frstar",
        d: "M 24 74 L 124 74 L 124 74 L 324 74 L 324 74 L 474 74 L 474 174 L 474 174 L 424 174 L 424 174 L 174 174 L 174 174 L 74 174 L 74 274 L 74 274 L 174 274 L 174 274 L 524 274",
        length: 1500,
        start: {x: 24, y: 74},
        dots: [{x: 124, y: 74}, {x: 324, y: 74}, {x: 474, y: 74}, {x: 424, y: 174}, {x: 174, y: 174}, {x: 74, y: 174}, {x: 174, y: 274}],
        waits: [
            {x: 124, y: 74, time: 20, dist: 100},
            {x: 324, y: 74, time: 20, dist: 300},
            {x: 474, y: 74, time: 20, dist: 450},
            {x: 424, y: 174, time: 20, dist: 600},
            {x: 174, y: 174, time: 20, dist: 850},
            {x: 74, y: 174, time: 20, dist: 950},
            {x: 174, y: 274, time: 20, dist: 1150},
        ],
        isAir: false,
    },
    {
        position: 1,
        timestamp: 16,
        enemyIndexRange: "2 ~ 3",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 2,
        timestamp: 23,
        enemyIndexRange: "4 ~ 5",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 3,
        timestamp: 32,
        enemyIndexRange: "6 ~ 7",
        enemyKey: "enemy_1029_shdsbr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 4,
        timestamp: 41,
        enemyIndexRange: "8 ~ 9",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 5,
        timestamp: 47,
        enemyIndexRange: "10 ~ 11",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 6,
        timestamp: 76,
        enemyIndexRange: "12 ~ 14",
        enemyKey: "enemy_1011_wizard_2",
        d: "M 24 174 L 524 174",
        length: 500,
        start: {x: 24, y: 174},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 7,
        timestamp: 79,
        enemyIndexRange: "15 ~ 16",
        enemyKey: "enemy_1034_laxe",
        d: "M 24 174 L 524 174",
        length: 500,
        start: {x: 24, y: 174},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 8,
        timestamp: 81,
        enemyIndexRange: "17 ~ 19",
        enemyKey: "enemy_1032_katar",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 9,
        timestamp: 82,
        enemyIndexRange: "20 ~ 22",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 174 274 L 174 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [{x: 174, y: 274}],
        waits: [],
        isAir: false,
    },
    {
        position: 10,
        timestamp: 142,
        enemyIndexRange: "23",
        enemyKey: "enemy_1006_shield_2",
        d: "M 24 174 L 524 174",
        length: 500,
        start: {x: 24, y: 174},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 11,
        timestamp: 142,
        enemyIndexRange: "24 ~ 25",
        enemyKey: "enemy_1034_laxe",
        d: "M 24 274 L 174 274 L 174 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [{x: 174, y: 274}],
        waits: [{x: 174, y: 274, time: 15, dist: 150}],
        isAir: false,
    },
    {
        position: 12,
        timestamp: 144,
        enemyIndexRange: "26 ~ 27",
        enemyKey: "enemy_1029_shdsbr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 13,
        timestamp: 152,
        enemyIndexRange: "28 ~ 29",
        enemyKey: "enemy_1011_wizard_2",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 14,
        timestamp: 158,
        enemyIndexRange: "30 ~ 31",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 15,
        timestamp: 200,
        enemyIndexRange: "32 ~ 34",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 174 L 524 174",
        length: 500,
        start: {x: 24, y: 174},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 16,
        timestamp: 200,
        enemyIndexRange: "35 ~ 37",
        enemyKey: "enemy_1002_nsabr",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 17,
        timestamp: 200.5,
        enemyIndexRange: "38 ~ 40",
        enemyKey: "enemy_1032_katar",
        d: "M 24 174 L 524 174",
        length: 500,
        start: {x: 24, y: 174},
        dots: [],
        waits: [],
        isAir: false,
    },
    {
        position: 18,
        timestamp: 200.5,
        enemyIndexRange: "41 ~ 43",
        enemyKey: "enemy_1032_katar",
        d: "M 24 274 L 524 274",
        length: 500,
        start: {x: 24, y: 274},
        dots: [],
        waits: [],
        isAir: false,
    },
];
const ROUTES = { entries: ROUTE_ENTRIES, total: 43, dims: { width: 548, height: 348 } };
const SETTINGS = { showRoutes: true, showEnemyIcons: false, showMovement: false, showTimers: true, walkingChibis: true };
const speedFor = () => 55;

const Stage = ({ focus, walkers, tilt }: { focus: number; walkers: unknown[]; tilt?: number }) => (
    <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px]">
        <div className="relative mx-auto" style={{ width: BOARD_W, height: BOARD_H }}>
            <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: BOARD_W, height: BOARD_H }}>
                <Board level={LEVEL} operators={new Map()} hovered coordOverride={null} />
                <RoutesLayer routes={ROUTES} focus={focus} hovered settings={SETTINGS} speedFor={speedFor} />
                <div className="transform-3d pointer-events-none absolute top-0 left-0 rotate-x-30 transition-transform duration-400 ease-[ease]" style={{ width: BOARD_W, height: BOARD_H }}>
                    <DynamicChibiLayer walkers={walkers} width={BOARD_W} height={BOARD_H} padY={48} tilt={tilt} />
                </div>
            </div>
        </div>
    </div>
);

/** The client-only wrapper MapView actually mounts: WebGL never runs on the server. */
export const OnTheBoard = () => <Stage focus={0} walkers={WALKERS} />;

/** Whatever `walkers` holds is what the lazy layer draws — one boss, one canvas. */
export const SingleWalker = () => <Stage focus={0} walkers={[WALKERS[0]]} />;

/** Empty `walkers` renders nothing at all; MapView skips mounting it in that case. */
export const NoWalkers = () => <Stage focus={11} walkers={[]} />;
