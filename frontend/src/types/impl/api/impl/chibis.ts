export type Chibis = {
    name: string;
    path: string;
    contentType: "directory" | "file";
    children: ChibiChildren[];
};

export type ChibisSimplified = {
    operatorCode: string;
    name: string;
    path: string;
    skins: Array<{
        name: string;
        path: string;
        hasSpineData: boolean;
        animationTypes?: {
            front?: {
                atlas: string;
                skel: string;
                png: string;
            };
            back?: {
                atlas: string;
                skel: string;
                png: string;
            };
            dorm?: {
                atlas: string;
                skel: string;
                png: string;
            };
        };
    }>;
};

export type Chibi = {
    name: string;
    path: string;
    contentType: "file" | "directory";
    children: ChibiChildren[];
};

type ChibiChildren = {
    name: string;
    path: string;
    contentType: "file" | "directory";
    children: ChibiChildren[];
};

export type ChibiOperatorList = string[];
