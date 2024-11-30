export type Favor = {
    maxFavor: number;
    favorFrames: {
        level: number;
        data: {
            favorPoint: number;
            percent: number;
            battlePhase: number;
        };
    }[];
};
