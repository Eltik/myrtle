export const MOTION_MODE = { WALK: "WALK", FLY: "FLY", E_NUM: "E_NUM" } as const;

export const MOVE_TYPE = {
    MOVE: "MOVE",
    WAIT_FOR_SECONDS: "WAIT_FOR_SECONDS",
    WAIT_FOR_PLAY_TIME: "WAIT_FOR_PLAY_TIME",
    WAIT_CURRENT_FRAGMENT_TIME: "WAIT_CURRENT_FRAGMENT_TIME",
    WAIT_CURRENT_WAVE_TIME: "WAIT_CURRENT_WAVE_TIME",
    DISAPPEAR: "DISAPPEAR",
    APPEAR_AT_POS: "APPEAR_AT_POS",
    ALERT: "ALERT",
    PATROL_MOVE: "PATROL_MOVE",
    INVALID: "INVALID",
} as const;

export const encodeHeightType = (v: string | number): number => {
    switch (v) {
        case "LOWLAND":
            return 0;
        case "HIGHLAND":
            return 1;
        default:
            return typeof v === "number" ? v : 0;
    }
};

export const encodeBuildableType = (v: string | number): number => {
    switch (v) {
        case "NONE":
            return 0;
        case "MELEE":
            return 1;
        case "RANGED":
            return 2;
        case "ALL":
            return 3;
        default:
            return typeof v === "number" ? v : 0;
    }
};

export const encodePassableMask = (v: string | number): number => {
    switch (v) {
        case "NONE":
            return 0;
        case "WALK_ONLY":
            return 1;
        case "FLY_ONLY":
            return 2;
        case "ALL":
            return 3;
        default:
            return typeof v === "number" ? v : 0;
    }
};

export const encodeMotionMode = (v: string | number): number => {
    switch (v) {
        case "WALK":
            return 0;
        case "FLY":
            return 1;
        default:
            return typeof v === "number" ? v : 0;
    }
};
