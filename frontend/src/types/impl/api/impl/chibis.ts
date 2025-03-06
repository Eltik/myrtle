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
    skins: {
        name: string;
        path: string;
        hasSpineData: boolean;
        hasCombatAnimations?: boolean;
        spineFiles: {
            atlas: string | null;
            skel: string | null;
            png: string | null;
            combat?: Array<{
                name: string;
                path: string;
                files?: {
                    atlas: string;
                    skel: string;
                    png: string;
                };
            }> | null;
        };
    }[];
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
