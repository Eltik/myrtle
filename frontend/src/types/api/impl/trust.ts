// Trust/Favor types

export interface FavorFrameData {
    favorPoint: number;
    percent: number;
    battlePhase: number;
}

export interface FavorFrame {
    level: number;
    data: FavorFrameData;
}

export interface Favor {
    maxFavor: number;
    favorFrames: FavorFrame[];
}
