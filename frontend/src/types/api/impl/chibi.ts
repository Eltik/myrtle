// Chibi Types - Spine animation data for operators

export interface SpineFiles {
    atlas: string | null;
    skel: string | null;
    png: string | null;
}

export interface ChibiSkin {
    name: string;
    path: string;
    hasSpineData: boolean;
    animationTypes: Record<string, SpineFiles>;
}

export interface ChibiCharacter {
    operatorCode: string;
    name: string;
    path: string;
    skins: ChibiSkin[];
}

export interface ChibiData {
    characters: ChibiCharacter[];
}

export type ChibiAnimationType = "front" | "back" | "dorm" | "dynamic";
