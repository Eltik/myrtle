export interface RepoItem {
    name: string;
    path: string;
    contentType: string;
    children?: RepoItem[];
}

export interface CachedData {
    timestamp: number;
    data: RepoItem[];
    version?: number;
}

// Animation types for different views
export type AnimationType = "front" | "back" | "dorm";

// Spine files for an animation type
export interface SpineFiles {
    atlas: string | null;
    skel: string | null;
    png: string | null;
}

// Character skin with different animation types
export interface CharacterSkin {
    name: string;
    path: string;
    hasSpineData: boolean;
    animationTypes: {
        front?: SpineFiles;
        back?: SpineFiles;
        dorm?: SpineFiles;
    };
}

// Processed character data for frontend
export interface CharacterData {
    operatorCode: string;
    name: string;
    path: string;
    skins: CharacterSkin[];
}
