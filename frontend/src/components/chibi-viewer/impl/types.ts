import type { Operator } from "~/types/impl/api/static/operator";

// Animation types
export type AnimationType = "dorm" | "front" | "back";

// Animation data for a specific view
export interface AnimationData {
    atlas: string;
    png: string;
    skel: string;
    path: string;
}

// Skin data for rendering
export interface FormattedSkin {
    name: string;
    dorm: AnimationData;
    front: AnimationData;
    back: AnimationData;
}

// Complete operator data with skins
export interface FormattedChibis {
    name: string;
    operatorCode: string;
    path: string;
    skins: FormattedSkin[];
    data?: Operator;
}

// Processed skin data for the renderer
export interface SkinData {
    atlas: string;
    png: string;
    skel: string;
    name: string;
    code: string;
    path: string;
    view: AnimationType;
}

// For the pixi-spine resource map type
export type ResourceMap = Record<string, {
    spineData?: unknown;
    texture?: unknown;
}>;

// For spine animation type
export interface SpineAnimation {
    name: string;
} 