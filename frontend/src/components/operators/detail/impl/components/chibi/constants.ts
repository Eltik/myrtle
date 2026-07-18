export const CHIBI_OFFSET_X = 0.5;
export const CHIBI_OFFSET_Y = 0.85;
export const CHIBI_SCALE = 0.75;
export const ANIMATION_SPEED = 1;

// Dynamic illustrations are full-size art (thousands of px), so they are
// fitted to the canvas from measured bounds instead of using the fixed chibi
// anchor/scale. The margin leaves a little breathing room at the edges.
export const DYNAMIC_FIT_MARGIN = 0.95;

export const EXPORT_WIDTH = 600;
export const EXPORT_HEIGHT = 400;
export const EXPORT_PADDING = 8;
// Dyn illust bounds can be huge; past this the H.264 encoder rejects the
// configure and gif.js memory explodes, so exports shrink to fit instead.
export const MAX_EXPORT_DIM = 2048;
export const EXPORT_BG_COLOR = 0x111014;

export type ViewType = "front" | "back" | "dorm" | "dynamic";

export interface IExportSettings {
    scale: number; // 0.5, 1, 1.5, 2
    fps: number; // 10, 15, 20, 30, 60
    transparentBg: boolean; // GIF only
    loopCount: number; // Minimum number of loops for video export
}

export const DEFAULT_EXPORT_SETTINGS: IExportSettings = {
    scale: 1,
    fps: 60,
    transparentBg: true,
    loopCount: 3,
};

export const EXPORT_SCALE_OPTIONS = [
    { value: 0.5, label: "300x200" },
    { value: 1, label: "600x400" },
    { value: 1.5, label: "900x600" },
    { value: 2, label: "1200x800" },
] as const;

export const EXPORT_FPS_OPTIONS = [10, 15, 20, 30, 60] as const;

export const EXPORT_LOOP_OPTIONS = [1, 2, 3, 5, 10] as const;
