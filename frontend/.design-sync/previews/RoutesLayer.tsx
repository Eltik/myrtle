import { Board, RoutesLayer } from "frontend";

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

/**
 * 4-10's real route geometry, as `buildRouteEntries` derives it from the level's
 * 20 route definitions and its single wave: entry 0 is FrostNova's teleporting
 * patrol, the rest are the mob lanes.
 */
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

const Surface = ({ children }: { children?: React.ReactNode }) => (
    <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818] bg-[linear-gradient(90deg,#0a0a0a_1.5px,transparent_1%),linear-gradient(#0a0a0a_1.5px,transparent_1%)] bg-position-[50%] bg-size-[2.5px_2.5px]">
        <div className="relative mx-auto" style={{ width: 548, height: 444 }}>
            <div className="perspective-[900px] perspective-origin-[50%] transform-3d absolute top-0 left-0" style={{ width: 548, height: 444 }}>
                {children}
            </div>
        </div>
    </div>
);

const NO_OPERATORS = new Map();

/** Focused on spawn 1: FrostNova's patrol, its wait timers and its per-checkpoint dots. */
export const FocusedBossPatrol = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered coordOverride={null} />
        <RoutesLayer routes={ROUTES} focus={0} hovered settings={SETTINGS} speedFor={speedFor} />
    </Surface>
);

/** With walking chibis off, `showEnemyIcons` badges the spawn point with the enemy's portrait. */
export const WithEnemyIcons = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered coordOverride={null} />
        <RoutesLayer routes={ROUTES} focus={0} hovered settings={{ ...SETTINGS, showEnemyIcons: true, walkingChibis: false }} speedFor={speedFor} />
    </Surface>
);

/** Timers off, board flattened: just the path and its waypoints. */
export const FlatWithoutTimers = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered={false} coordOverride={null} />
        <RoutesLayer routes={ROUTES} focus={0} hovered={false} settings={{ ...SETTINGS, showTimers: false }} speedFor={speedFor} />
    </Surface>
);

/** A later spawn group — the counter tracks which of the stage's 43 enemies is shown. */
export const LaterSpawnGroup = () => (
    <Surface>
        <Board level={LEVEL} operators={NO_OPERATORS} hovered coordOverride={null} />
        <RoutesLayer routes={ROUTES} focus={11} hovered settings={SETTINGS} speedFor={speedFor} />
    </Surface>
);
