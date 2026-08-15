import { SpawnSchedule } from "frontend";

const ENEMIES = {
    enemy_1034_laxe: {enemyId: "enemy_1034_laxe", name: "Butcher", enemyLevel: "ELITE"},
    enemy_1032_katar: {enemyId: "enemy_1032_katar", name: "Bladed Fighter", enemyLevel: "NORMAL"},
    enemy_1006_shield_2: {enemyId: "enemy_1006_shield_2", name: "Heavy Defender Leader", enemyLevel: "ELITE"},
    enemy_1505_frstar: {enemyId: "enemy_1505_frstar", name: "FrostNova", enemyLevel: "BOSS"},
    enemy_1002_nsabr: {enemyId: "enemy_1002_nsabr", name: "Soldier", enemyLevel: "NORMAL"},
    enemy_1029_shdsbr: {enemyId: "enemy_1029_shdsbr", name: "Shielded Soldier", enemyLevel: "NORMAL"},
    enemy_1011_wizard_2: {enemyId: "enemy_1011_wizard_2", name: "Caster Leader", enemyLevel: "NORMAL"},
};
/** 4-10's single wave: five fragments, 20 SPAWN actions, 43 enemies. */
const WAVES = [
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
];
const ENEMY_DB_REFS = [
    {id: "enemy_1505_frstar", level: 0, useDb: true},
    {id: "enemy_1002_nsabr", level: 1, useDb: true},
    {id: "enemy_1032_katar", level: 0, useDb: true},
    {id: "enemy_1029_shdsbr", level: 0, useDb: true},
    {id: "enemy_1034_laxe", level: 0, useDb: true},
    {id: "enemy_1006_shield_2", level: 0, useDb: true},
    {id: "enemy_1011_wizard_2", level: 0, useDb: true},
];

const LEVEL = { waves: WAVES, enemyDbRefs: ENEMY_DB_REFS };
const noop = () => {};

/** Every spawn group in 4-10 — Extinguished Flames, in firing order. */
export const FullSchedule = () => <SpawnSchedule level={LEVEL} enemyData={ENEMIES} onFocusEnemy={noop} />;

/** The opening fragments only — a short stage fills a single screen. */
export const OpeningFragments = () => <SpawnSchedule level={{ ...LEVEL, waves: [{ ...WAVES[0], fragments: WAVES[0].fragments.slice(0, 2) }] }} enemyData={ENEMIES} onFocusEnemy={noop} />;

/** With no handbook data the rows fall back to the raw enemy id and the action type. */
export const UnresolvedEnemies = () => <SpawnSchedule level={{ ...LEVEL, waves: [{ ...WAVES[0], fragments: WAVES[0].fragments.slice(0, 2) }] }} enemyData={{}} onFocusEnemy={noop} />;
