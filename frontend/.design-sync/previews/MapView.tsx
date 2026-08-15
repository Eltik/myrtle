import { useEffect, useRef } from "react";
import { MapView } from "frontend";

/** 4-10 — Extinguished Flames, as `GET /level/main_04-10` ships it. */
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
    routes: [
    {motionMode: "E_NUM", startPosition: {row: 0, col: 0}, endPosition: {row: 0, col: 0}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {
        motionMode: "WALK",
        startPosition: {row: 5, col: 0},
        endPosition: {row: 1, col: 10},
        checkpoints: [
            {type: "MOVE", position: {row: 5, col: 2}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "MOVE", position: {row: 5, col: 6}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "MOVE", position: {row: 5, col: 9}},
            {type: "DISAPPEAR", position: {row: 0, col: 0}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "APPEAR_AT_POS", position: {row: 3, col: 9}},
            {type: "MOVE", position: {row: 3, col: 8}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "MOVE", position: {row: 3, col: 3}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "MOVE", position: {row: 3, col: 1}},
            {type: "DISAPPEAR", position: {row: 0, col: 0}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
            {type: "APPEAR_AT_POS", position: {row: 1, col: 1}},
            {type: "MOVE", position: {row: 1, col: 3}},
            {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 20},
        ],
        allowDiagonalMove: true,
        visitEveryCheckPoint: false,
    },
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 3, col: 0}, endPosition: {row: 3, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 3, col: 0}, endPosition: {row: 3, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {
        motionMode: "WALK",
        startPosition: {row: 1, col: 0},
        endPosition: {row: 1, col: 10},
        checkpoints: [{type: "MOVE", position: {row: 1, col: 3}}],
        allowDiagonalMove: true,
        visitEveryCheckPoint: false,
    },
    {motionMode: "WALK", startPosition: {row: 3, col: 0}, endPosition: {row: 3, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {
        motionMode: "WALK",
        startPosition: {row: 1, col: 0},
        endPosition: {row: 1, col: 10},
        checkpoints: [{type: "MOVE", position: {row: 1, col: 3}}, {type: "WAIT_FOR_SECONDS", position: {row: 0, col: 0}, time: 15}],
        allowDiagonalMove: true,
        visitEveryCheckPoint: false,
    },
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 3, col: 0}, endPosition: {row: 3, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 3, col: 0}, endPosition: {row: 3, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
    {motionMode: "WALK", startPosition: {row: 1, col: 0}, endPosition: {row: 1, col: 10}, checkpoints: [], allowDiagonalMove: true, visitEveryCheckPoint: false},
],
    waves: [
    {
        preDelay: 0,
        postDelay: 0,
        maxTimeWaitingForNextWave: -1,
        fragments: [
            {
                preDelay: 0,
                actions: [
                    {actionType: "DISPLAY_ENEMY_INFO", key: "enemy_1505_frstar", routeIndex: 0, count: 1, interval: 1, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1505_frstar", routeIndex: 1, count: 1, interval: 1, preDelay: 3},
                ],
            },
            {
                preDelay: 10,
                actions: [
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 2, count: 2, interval: 1, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 3, count: 2, interval: 1, preDelay: 10},
                    {actionType: "SPAWN", key: "enemy_1029_shdsbr", routeIndex: 4, count: 2, interval: 1, preDelay: 19},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 5, count: 2, interval: 1, preDelay: 28},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 6, count: 2, interval: 1, preDelay: 34},
                ],
            },
            {
                preDelay: 25,
                actions: [
                    {actionType: "SPAWN", key: "enemy_1011_wizard_2", routeIndex: 7, count: 3, interval: 12, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1034_laxe", routeIndex: 8, count: 2, interval: 25, preDelay: 6},
                    {actionType: "SPAWN", key: "enemy_1032_katar", routeIndex: 9, count: 3, interval: 10, preDelay: 8},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 10, count: 3, interval: 10, preDelay: 9},
                ],
            },
            {
                preDelay: 35,
                actions: [
                    {actionType: "SPAWN", key: "enemy_1006_shield_2", routeIndex: 11, count: 1, interval: 1, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1034_laxe", routeIndex: 12, count: 2, interval: 30, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1029_shdsbr", routeIndex: 13, count: 2, interval: 1, preDelay: 5},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 14, count: 2, interval: 1, preDelay: 19},
                    {actionType: "SPAWN", key: "enemy_1011_wizard_2", routeIndex: 15, count: 2, interval: 18, preDelay: 13},
                ],
            },
            {
                preDelay: 25,
                actions: [
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 16, count: 3, interval: 20, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1032_katar", routeIndex: 17, count: 3, interval: 20, preDelay: 3.5},
                    {actionType: "SPAWN", key: "enemy_1002_nsabr", routeIndex: 18, count: 3, interval: 20, preDelay: 3},
                    {actionType: "SPAWN", key: "enemy_1032_katar", routeIndex: 19, count: 3, interval: 20, preDelay: 3.5},
                ],
            },
        ],
    },
],
    predefines: { tokenInsts: [] },
    enemyDbRefs: [
    {id: "enemy_1505_frstar", level: 0, useDb: true},
    {id: "enemy_1002_nsabr", level: 1, useDb: true},
    {id: "enemy_1032_katar", level: 0, useDb: true},
    {id: "enemy_1029_shdsbr", level: 0, useDb: true},
    {id: "enemy_1034_laxe", level: 0, useDb: true},
    {id: "enemy_1006_shield_2", level: 0, useDb: true},
    {id: "enemy_1011_wizard_2", level: 0, useDb: true},
],
    tilesDisallowToLocate: [],
    options: {
    characterLimit: 10,
    maxLifePoint: 3,
    initialCost: 10,
    maxCost: 99,
    costIncreaseTime: 1,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
},
};

const ENEMIES = {
    enemy_1034_laxe: {
        enemyId: "enemy_1034_laxe",
        name: "Butcher",
        enemyLevel: "ELITE",
        stats: {
            levels: [
                {
                    level: 0,
                    attributes: {maxHp: 9000, atk: 850, def: 230, magicResistance: 30, moveSpeed: 0.699999988079071, attackSpeed: 100, baseAttackTime: 3.5},
                },
            ],
        },
    },
    enemy_1032_katar: {
        enemyId: "enemy_1032_katar",
        name: "Bladed Fighter",
        enemyLevel: "NORMAL",
        stats: {levels: [{level: 0, attributes: {maxHp: 4000, atk: 450, def: 300, magicResistance: 0, moveSpeed: 1, attackSpeed: 100, baseAttackTime: 1.5}}]},
    },
    enemy_1006_shield_2: {
        enemyId: "enemy_1006_shield_2",
        name: "Heavy Defender Leader",
        enemyLevel: "ELITE",
        stats: {
            levels: [
                {
                    level: 0,
                    attributes: {maxHp: 10000, atk: 600, def: 1000, magicResistance: 0, moveSpeed: 0.75, attackSpeed: 100, baseAttackTime: 2.5999999046325684},
                },
                {
                    level: 1,
                    attributes: {maxHp: 15000, atk: 700, def: 850, magicResistance: 0, moveSpeed: 0.75, attackSpeed: 100, baseAttackTime: 2.5999999046325684},
                },
            ],
        },
    },
    enemy_1505_frstar: {
        enemyId: "enemy_1505_frstar",
        name: "FrostNova",
        enemyLevel: "BOSS",
        stats: {
            levels: [
                {
                    level: 0,
                    attributes: {maxHp: 25000, atk: 420, def: 250, magicResistance: 50, moveSpeed: 0.5, attackSpeed: 100, baseAttackTime: 3.700000047683716},
                },
                {
                    level: 1,
                    attributes: {maxHp: 35000, atk: 620, def: 350, magicResistance: 50, moveSpeed: 0.5, attackSpeed: 100, baseAttackTime: 3.700000047683716},
                },
            ],
        },
    },
    enemy_1002_nsabr: {
        enemyId: "enemy_1002_nsabr",
        name: "Soldier",
        enemyLevel: "NORMAL",
        stats: {
            levels: [
                {level: 0, attributes: {maxHp: 1650, atk: 200, def: 100, magicResistance: 0, moveSpeed: 1.100000023841858, attackSpeed: 100, baseAttackTime: 2}},
                {level: 1, attributes: {maxHp: 2750, atk: 300, def: 130, magicResistance: 0, moveSpeed: 1.100000023841858, attackSpeed: 100, baseAttackTime: 2}},
            ],
        },
    },
    enemy_1029_shdsbr: {
        enemyId: "enemy_1029_shdsbr",
        name: "Shielded Soldier",
        enemyLevel: "NORMAL",
        stats: {levels: [{level: 0, attributes: {maxHp: 2050, atk: 240, def: 250, magicResistance: 0, moveSpeed: 1, attackSpeed: 100, baseAttackTime: 2}}]},
    },
    enemy_1011_wizard_2: {
        enemyId: "enemy_1011_wizard_2",
        name: "Caster Leader",
        enemyLevel: "NORMAL",
        stats: {
            levels: [
                {level: 0, attributes: {maxHp: 2400, atk: 300, def: 80, magicResistance: 50, moveSpeed: 0.800000011920929, attackSpeed: 100, baseAttackTime: 4}},
                {
                    level: 1,
                    attributes: {maxHp: 2300, atk: 420, def: 100, magicResistance: 50, moveSpeed: 0.800000011920929, attackSpeed: 100, baseAttackTime: 4},
                },
                {
                    level: 2,
                    attributes: {maxHp: 6500, atk: 600, def: 500, magicResistance: 50, moveSpeed: 0.800000011920929, attackSpeed: 100, baseAttackTime: 4},
                },
            ],
        },
    },
};
const STATS = {
    enemy_1505_frstar: {levelIndex: 0, phaseCount: 2, maxHp: 25000, atk: 420, def: 250, res: 50, moveSpeed: 0.25, attackInterval: 3.700000047683716, hasOverride: false},
    enemy_1002_nsabr: {levelIndex: 1, phaseCount: 2, maxHp: 2750, atk: 300, def: 130, res: 0, moveSpeed: 0.550000011920929, attackInterval: 2, hasOverride: false},
    enemy_1032_katar: {levelIndex: 0, phaseCount: 1, maxHp: 4000, atk: 450, def: 300, res: 0, moveSpeed: 0.5, attackInterval: 1.5, hasOverride: false},
    enemy_1029_shdsbr: {levelIndex: 0, phaseCount: 1, maxHp: 2050, atk: 240, def: 250, res: 0, moveSpeed: 0.5, attackInterval: 2, hasOverride: false},
    enemy_1034_laxe: {levelIndex: 0, phaseCount: 1, maxHp: 9000, atk: 850, def: 230, res: 30, moveSpeed: 0.3499999940395355, attackInterval: 3.5, hasOverride: false},
    enemy_1006_shield_2: {levelIndex: 0, phaseCount: 2, maxHp: 10000, atk: 600, def: 1000, res: 0, moveSpeed: 0.375, attackInterval: 2.5999999046325684, hasOverride: false},
    enemy_1011_wizard_2: {levelIndex: 0, phaseCount: 3, maxHp: 2400, atk: 300, def: 80, res: 50, moveSpeed: 0.4000000059604645, attackInterval: 4, hasOverride: false},
};
const statsFor = (id: string) => STATS[id as keyof typeof STATS] ?? null;

const DEFAULT_SETTINGS = { showRoutes: true, showEnemyIcons: false, showMovement: false, showTimers: true, walkingChibis: true };

/**
 * The map opens with nothing focused; a doctor steps through spawns with the
 * arrow buttons. Driving the imperative handle on mount is what a preview needs
 * to show a focused route, and it has to wait two frames for the board to lay out.
 */
function FocusedMap({ enemyId, time, settings = DEFAULT_SETTINGS }: { enemyId: string; time?: number; settings?: typeof DEFAULT_SETTINGS }) {
    const ref = useRef<{ focusSpawn: (t: { enemyId: string; time?: number }) => boolean } | null>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => ref.current?.focusSpawn({ enemyId, time }));
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, [enemyId, time]);
    return (
        <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818]">
            <MapView ref={ref} code="4-10" enemyData={ENEMIES} level={LEVEL} settings={settings} statsFor={statsFor} />
        </div>
    );
}

/** The initial state: the board in 3D, no spawn focused yet. */
export const BossStageBoard = () => (
    <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818]">
        <MapView code="4-10" enemyData={ENEMIES} level={LEVEL} settings={DEFAULT_SETTINGS} statsFor={statsFor} />
    </div>
);

/** Stepping to FrostNova's spawn draws her teleporting patrol and its wait timers. */
export const FocusedBossRoute = () => <FocusedMap enemyId="enemy_1505_frstar" time={3} />;

/** A mid-stage mob group, with route lines and timers left on. */
export const FocusedMobLane = () => <FocusedMap enemyId="enemy_1034_laxe" time={142} />;

/** Route lines and timers off: only the board and the enemy overlay remain. */
export const OverlaysOff = () => <FocusedMap enemyId="enemy_1505_frstar" time={3} settings={{ ...DEFAULT_SETTINGS, showRoutes: false, showTimers: false }} />;

/** A stage with no level file renders nothing — the detail page shows its own empty panel. */
export const NoLevelData = () => (
    <div className="flex min-h-70 items-center justify-center rounded-[14px] border border-border border-dashed p-10 text-center">
        <MapView level={null} />
        <p className="m-0 font-sans text-[13px] text-muted-foreground">No map data available for this stage.</p>
    </div>
);
