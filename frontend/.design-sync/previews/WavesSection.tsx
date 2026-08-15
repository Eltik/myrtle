import { WavesSection } from "frontend";

/** 4-10 — Extinguished Flames ships one wave of five fragments. */
const WAVES_4_10 = [
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

/**
 * Ursus — Frozen Abandoned City (`camp_r_02`), an Annihilation map: five waves.
 * Each fragment is listed as the SPAWN counts it fires, which is all the
 * section totals.
 */
const ANNIHILATION_SPEC = [
    {
        preDelay: 0,
        postDelay: 0,
        frags: [
            [1, 1, 2, 2, 2, 1, 1, 1],
            [1, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 1],
            [3, 2, 2, 2, 3, 2, 2, 2, 2],
            [1, 1, 2, 2, 2, 1, 2, 2],
            [1, 1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 1, 2, 2],
            [1, 1, 1, 1, 1, 1, 3, 1, 2, 1, 2, 1],
        ],
    },
    {
        preDelay: 5,
        postDelay: 0,
        frags: [
            [1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1],
            [8, 7, 5],
            [1, 1, 1, 1, 1, 1, 2, 2, 3, 2, 2, 2, 1],
            [2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1, 3, 1, 1, 1, 1],
        ],
    },
    {
        preDelay: 5,
        postDelay: 0,
        frags: [
            [1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 2, 2, 2, 1, 1, 2, 1, 1],
            [1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1],
            [3, 3, 2, 2, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 1, 1, 1, 1, 3, 1, 1, 2],
        ],
    },
    {
        preDelay: 5,
        postDelay: 0,
        frags: [
            [1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 1, 2],
            [1, 1, 1, 2, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 2, 2, 2],
        ],
    },
    {preDelay: 5, postDelay: 0, frags: [[1, 2, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2]]},
];
const ANNIHILATION_WAVES = ANNIHILATION_SPEC.map((w) => ({
    preDelay: w.preDelay,
    postDelay: w.postDelay,
    fragments: w.frags.map((counts) => ({ preDelay: 0, actions: counts.map((count) => ({ actionType: "SPAWN", preDelay: 0, count })) })),
}));

export const SingleWave = () => (
    <div className="max-w-md">
        <WavesSection level={{ waves: WAVES_4_10 }} />
    </div>
);

export const AnnihilationWaves = () => (
    <div className="max-w-md">
        <WavesSection level={{ waves: ANNIHILATION_WAVES.slice(0, 4) }} />
    </div>
);

/** A short reinforcement wave: one fragment, a long post-delay before the next. */
export const ShortWave = () => (
    <div className="max-w-md">
        <WavesSection level={{ waves: ANNIHILATION_WAVES.slice(4) }} />
    </div>
);
