export enum AssetServers {
    OFFICIAL = 0,
    BILIBILI = 1,
}

export interface AssetInfo {
    totalSize: number;
    abSize: number;
    md5: string;
}

interface PackageInfo {
    totalSize: number;
    files: Record<string, AssetInfo>;
}

export interface HotUpdateList {
    [key: string]: PackageInfo;
}

// Logger utility for better debugging
export enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
}

// Types for Unity asset bundle parsing
export interface UnityBundleHeader {
    signature: string;
    version: number;
    unityVersion: string;
    unityRevision: string;
    size: bigint;
    compressedBlocksInfoSize: number;
    uncompressedBlocksInfoSize: number;
    flags: number;
}

export interface UnityAsset {
    type: string;
    format?: string;
    offset: number;
    size: number;
    name: string;
}
