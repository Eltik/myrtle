// Zone types for stage randomization

export type ZoneType = "MAINLINE" | "SIDESTORY" | "BRANCHLINE" | "ACTIVITY" | "WEEKLY" | "CAMPAIGN" | "CLIMB_TOWER" | "ROGUELIKE" | "GUIDE" | "EVOLVE" | "MAINLINE_ACTIVITY" | "MAINLINE_RETRO" | "SPECIAL";

export interface Zone {
    zoneId: string;
    zoneIndex: number;
    type: ZoneType;
    zoneNameFirst?: string;
    zoneNameSecond?: string;
    zoneNameTitleCurrent?: string;
    zoneNameTitleUnCurrent?: string;
    zoneNameTitleEx?: string;
    zoneNameThird?: string;
    lockedText?: string;
    canPreview: boolean;
    hasAdditionalPanel: boolean;
}

export interface ZonesResponse {
    zones: Zone[];
    nextCursor: string | null;
    hasMore: boolean;
    total: number;
}

export interface ZoneResponse {
    zone: Zone;
}
