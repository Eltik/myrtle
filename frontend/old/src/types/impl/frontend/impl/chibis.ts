import type { Operator } from "../../api/static/operator";

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
    data?: Operator;
};

// Add this interface for spine animations
export interface SpineAnimation {
    name: string;
}

// Define resource interface to help with type safety
export type ResourceMap = Record<
    string,
    {
        spineData?: unknown;
        data?: unknown;
    }
>;

export type AnimationType = {
    atlas: string;
    png: string;
    skel: string;
};

export type SkinData = {
    name: string;
    dorm?: { atlas: string; png: string; skel: string; path: string };
    front?: { atlas: string; png: string; skel: string; path: string };
    back?: { atlas: string; png: string; skel: string; path: string };
};

export type ChibiAnimation = "front" | "back" | "dorm";
