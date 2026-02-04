export const CHIBI_OFFSET_X = 0.5;
export const CHIBI_OFFSET_Y = 0.85;
export const CHIBI_SCALE = 0.75;
export const ANIMATION_SPEED = 0.5;

export const EXPORT_WIDTH = 600;
export const EXPORT_HEIGHT = 400;
export const EXPORT_GIF_FPS = 20;
export const EXPORT_MP4_FPS = 30;
export const EXPORT_BG_COLOR = 0x111014;

export type ViewType = "front" | "back" | "dorm";

export interface ExportSettings {
    scale: number; // 0.5, 1, 1.5, 2
    fps: number; // 10, 15, 20, 30, 60
    transparentBg: boolean; // GIF only
}

export const DEFAULT_EXPORT_SETTINGS: ExportSettings = {
    scale: 1,
    fps: 20,
    transparentBg: true,
};

export const EXPORT_SCALE_OPTIONS = [
    { value: 0.5, label: "300x200" },
    { value: 1, label: "600x400" },
    { value: 1.5, label: "900x600" },
    { value: 2, label: "1200x800" },
] as const;

export const EXPORT_FPS_OPTIONS = [10, 15, 20, 30, 60] as const;
