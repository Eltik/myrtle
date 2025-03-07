export type FormattedChibis = {
    name: string;
    operatorCode: string;
    path: string;
    skins: {
        name: string;
        dorm: {
            path: string;
            atlas: string;
            png: string;
            skel: string;
        };
        front: {
            path: string;
            atlas: string;
            png: string;
            skel: string;
        };
        back: {
            path: string;
            atlas: string;
            png: string;
            skel: string;
        };
    }[];
};
