import { type IChibiCharacter, type IChibiSkin, type IChibiSpineFiles, isCompleteSpineFiles } from "#/lib/api/chibis";
import type { ViewType } from "./constants";

// Kept free of pixi.js so route chunks that only need skin lookup or the fit default
// do not pull the renderer in statically.

const VIEW_FALLBACK_ORDER: readonly ViewType[] = ["front", "dorm", "back"] as const;

/** Requested skin, else "default", else the first available. */
function resolveSkin(chibi: IChibiCharacter, skinName: string): IChibiSkin | undefined {
    const skins = chibi.skins;
    if (skins.length === 0) return undefined;

    const nameLower = skinName.toLowerCase();
    let exact: IChibiSkin | undefined;
    let fallback: IChibiSkin | undefined;

    for (const skin of skins) {
        const lower = skin.name.toLowerCase();
        if (lower === nameLower) {
            exact = skin;
            break;
        }
        if (!fallback && lower === "default") fallback = skin;
    }

    return exact ?? fallback ?? skins[0];
}

export interface IResolvedChibiView {
    files: IChibiSpineFiles;
    view: ViewType;
}

/**
 * Resolve the spine files for a requested view, falling back to the chibi
 * views when it's unavailable. Returns which view was actually resolved so
 * the caller can pick the matching layout ("dynamic" is full-size art and is
 * never used as a fallback for chibi views).
 */
export function resolveChibiView(chibi: IChibiCharacter, skinName: string, viewType: ViewType): IResolvedChibiView | null {
    const skin = resolveSkin(chibi, skinName);
    if (!skin) return null;

    const types = skin.animationTypes;
    const requested = types[viewType];
    if (isCompleteSpineFiles(requested)) return { files: requested, view: viewType };

    for (const fallbackType of VIEW_FALLBACK_ORDER) {
        const fallback = types[fallbackType];
        if (isCompleteSpineFiles(fallback)) return { files: fallback, view: fallbackType };
    }

    return null;
}

export function getChibiSkinData(chibi: IChibiCharacter, skinName: string, viewType: ViewType): IChibiSpineFiles | null {
    return resolveChibiView(chibi, skinName, viewType)?.files ?? null;
}

export type SpineFitMode = "contain" | "cover" | "height";
export type SpineAlign = "top" | "center" | "bottom";
export interface ISpineFit {
    /**
     * `contain` = whole illustration visible (letterboxed); `cover` = fill the box,
     * cropping overflow; `height` = scale to the canvas HEIGHT only (the box's on-screen
     * height always equals the canvas height; excess width shows more scene, narrow width
     * crops the sides). `height` makes the subject fill a constant fraction of the frame
     * height regardless of the container's aspect ratio; the dynchar viewer uses it so the
     * character reads the same size on a tall mobile card, a wide desktop card, and a
     * fullscreen dialog alike (`cover` on a square box would inflate the subject with aspect).
     */
    mode: SpineFitMode;
    align: SpineAlign;
}

export const DEFAULT_SPINE_FIT: ISpineFit = { mode: "contain", align: "center" };

export function getAvailableViewTypes(chibi: IChibiCharacter | null, skinName: string | null): ViewType[] {
    if (!chibi || !skinName) return [];
    const skin = resolveSkin(chibi, skinName);
    if (!skin) return [];

    const types = skin.animationTypes;
    const result: ViewType[] = [];
    // Only list views whose spine assets are all present (skel + atlas + png);
    // a skel-only view would appear in the dropdown yet silently fall back on load.
    if (isCompleteSpineFiles(types.front)) result.push("front");
    if (isCompleteSpineFiles(types.back)) result.push("back");
    if (isCompleteSpineFiles(types.dorm)) result.push("dorm");
    // "down" is the third battle facing, and only three summon tokens have one, so it is
    // listed when present rather than assumed. It stays out of VIEW_FALLBACK_ORDER: a token
    // that has it always has a front too, and falling back to a downed pose would be wrong.
    if (isCompleteSpineFiles(types.down)) result.push("down");
    return result;
}
