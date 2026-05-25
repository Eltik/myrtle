export const MAX_PLAYER_LEVEL = 120;

// Required EXP at level L to advance to L+1, indexed by current level (1..119).
// Source: arknights.wiki.gg/wiki/EXP (Doctor level table).
const EXP_TO_NEXT: Record<number, number> = {
    1: 500,
    2: 800,
    3: 1240,
    4: 1320,
    5: 1400,
    6: 1480,
    7: 1560,
    8: 1640,
    9: 1720,
    10: 1800,
    11: 1880,
    12: 1960,
    13: 2040,
    14: 2120,
    15: 2200,
    16: 2280,
    17: 2360,
    18: 2440,
    19: 2520,
    20: 2600,
    21: 2680,
    22: 2760,
    23: 2840,
    24: 2920,
    25: 3000,
    26: 3080,
    27: 3160,
    28: 3240,
    29: 3350,
    30: 3460,
    31: 3570,
    32: 3680,
    33: 3790,
    34: 3900,
    35: 4200,
    36: 4500,
    37: 4800,
    38: 5100,
    39: 5400,
    40: 5700,
    41: 6000,
    42: 6300,
    43: 6600,
    44: 6900,
    45: 7200,
    46: 7500,
    47: 7800,
    48: 8100,
    49: 8400,
    50: 8700,
    51: 9000,
    52: 9500,
    53: 10000,
    54: 10500,
    55: 11000,
    56: 11500,
    57: 12000,
    58: 12500,
    59: 13000,
    60: 13500,
    61: 14000,
    62: 14500,
    63: 15000,
    64: 15500,
    65: 16000,
    66: 17000,
    67: 18000,
    68: 19000,
    69: 20000,
    70: 21000,
    71: 22000,
    72: 23000,
    73: 24000,
    74: 25000,
    75: 26000,
    76: 27000,
    77: 28000,
    78: 29000,
    79: 30000,
    80: 31000,
    81: 32000,
    82: 33000,
    83: 34000,
    84: 35000,
    85: 36000,
    86: 37000,
    87: 38000,
    88: 39000,
    89: 40000,
    90: 41000,
    91: 42000,
    92: 43000,
    93: 44000,
    94: 45000,
    95: 46000,
    96: 47000,
    97: 48000,
    98: 49000,
    99: 50000,
    100: 51000,
    101: 52000,
    102: 54000,
    103: 56000,
    104: 58000,
    105: 60000,
    106: 62000,
    107: 64000,
    108: 66000,
    109: 68000,
    110: 70000,
    111: 73000,
    112: 76000,
    113: 79000,
    114: 82000,
    115: 85000,
    116: 88000,
    117: 91000,
    118: 94000,
    119: 97000,
};

export function expForNextLevel(level: number): number | null {
    if (level >= MAX_PLAYER_LEVEL) return null;
    return EXP_TO_NEXT[level] ?? null;
}

export interface ILevelProgress {
    level: number;
    isMax: boolean;
    currentExp: number;
    requiredExp: number | null;
    ratio: number;
}

export function getLevelProgress(level: number | null, exp: number | null): ILevelProgress | null {
    if (level === null || level <= 0) return null;
    const clamped = Math.min(level, MAX_PLAYER_LEVEL);
    const isMax = clamped >= MAX_PLAYER_LEVEL;
    const requiredExp = expForNextLevel(clamped);
    const currentExp = Math.max(0, exp ?? 0);
    const ratio = isMax || !requiredExp ? 1 : Math.min(1, currentExp / requiredExp);
    return { level: clamped, isMax, currentExp, requiredExp, ratio };
}
