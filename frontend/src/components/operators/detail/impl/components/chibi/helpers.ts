import * as PIXI from "pixi.js";
import type { Spine } from "pixi-spine";
import { env } from "#/env";
import { type IChibiCharacter, type IChibiSkin, type IChibiSpineFiles, isCompleteSpineFiles } from "#/lib/api/chibis";
import { loadImage } from "#/lib/utils";
import { patchBakedIkRedundancy } from "./bakedIkFix";
import { CHIBI_OFFSET_X, CHIBI_OFFSET_Y, CHIBI_SCALE, DYNAMIC_FIT_MARGIN, EXPORT_HEIGHT, EXPORT_PADDING, EXPORT_WIDTH, MAX_EXPORT_DIM, type ViewType } from "./constants";
import { patchSpine38PathConstraint } from "./pathConstraintFix";

let spineModulesPromise: Promise<{
    Spine: typeof import("pixi-spine").Spine;
    TextureAtlas: typeof import("pixi-spine").TextureAtlas;
    AtlasAttachmentLoader: typeof import("@pixi-spine/runtime-3.8").AtlasAttachmentLoader;
    SkeletonBinary: typeof import("@pixi-spine/runtime-3.8").SkeletonBinary;
    SkeletonJson: typeof import("@pixi-spine/runtime-3.8").SkeletonJson;
}> | null = null;

function loadSpineModules() {
    if (!spineModulesPromise) {
        spineModulesPromise = Promise.all([import("pixi-spine"), import("@pixi-spine/runtime-3.8")]).then(([spine, runtime]) => ({
            Spine: spine.Spine,
            TextureAtlas: spine.TextureAtlas,
            AtlasAttachmentLoader: runtime.AtlasAttachmentLoader,
            SkeletonBinary: runtime.SkeletonBinary,
            SkeletonJson: runtime.SkeletonJson,
        }));
    }
    return spineModulesPromise;
}

const VIEW_FALLBACK_ORDER: readonly ViewType[] = ["front", "dorm", "back"] as const;

// Matches an atlas page declaration: a line ending in ".png" followed by "size: W,H".
// Capturing groups: 1=page name, 2=width, 3=height.
const ATLAS_PAGE_RE = /^(.+\.png)\r?\n\s*size:\s*(\d+)\s*,\s*(\d+)/gm;

function encodeAssetPath(path: string): string {
    if (!path) return "";
    if (/^[A-Za-z0-9\-_.!~*'()/%]*$/.test(path)) return path;

    let result = "";
    let segmentStart = 0;
    for (let i = 0; i <= path.length; i++) {
        if (i === path.length || path[i] === "/") {
            const segment = path.slice(segmentStart, i);
            // Preserve already-encoded segments (containing %), encode otherwise.
            result += segment.includes("%") ? segment : encodeURIComponent(segment);
            if (i < path.length) result += "/";
            segmentStart = i + 1;
        }
    }
    return result;
}

export function chibiAssetURL(path: string, server?: "en" | "cn"): string {
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${env.VITE_BACKEND_URL}/api/${prefix}assets${encodeAssetPath(path)}`;
}

/**
 * Build a BaseTexture from an image. If the actual PNG is smaller than the
 * atlas-declared size, the image is upscaled onto a canvas so atlas UV
 * coordinates remain valid (the game ships downscaled textures but the atlas
 * still references full-resolution coordinates).
 */
function buildPageTexture(img: HTMLImageElement, declaredW: number, declaredH: number): PIXI.BaseTexture {
    if (img.naturalWidth >= declaredW && img.naturalHeight >= declaredH) {
        return PIXI.BaseTexture.from(img);
    }

    const canvas = document.createElement("canvas");
    canvas.width = declaredW;
    canvas.height = declaredH;
    const ctx = canvas.getContext("2d");
    if (ctx) {
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = "high";
        ctx.drawImage(img, 0, 0, declaredW, declaredH);
    }
    return PIXI.BaseTexture.from(canvas);
}

/**
 * Parse atlas text and return a map of page filename → declared size.
 */
function parseAtlasPages(atlasText: string): Map<string, { declaredW: number; declaredH: number }> {
    const pages = new Map<string, { declaredW: number; declaredH: number }>();
    ATLAS_PAGE_RE.lastIndex = 0;
    let match = ATLAS_PAGE_RE.exec(atlasText);
    while (match !== null) {
        pages.set(match[1].trim(), {
            declaredW: Number.parseInt(match[2], 10),
            declaredH: Number.parseInt(match[3], 10),
        });
        match = ATLAS_PAGE_RE.exec(atlasText);
    }
    return pages;
}

export async function loadSpineWithEncodedURLs(skelPath: string, atlasPath: string, server?: "en" | "cn"): Promise<Spine> {
    const skelURL = chibiAssetURL(skelPath, server);
    const atlasURL = chibiAssetURL(atlasPath, server);

    // In dev, cache-bust the skel/atlas/page-image fetches so re-extracted spine
    // assets are picked up immediately. Without this, the backend's long
    // Cache-Control serves a stale page image; if it's smaller than the atlas's
    // declared size, buildPageTexture silently upscales it, mapping every region
    // to the wrong pixels (garbage). No-op in prod.
    const bust = import.meta.env.DEV ? `?v=${Date.now()}` : "";

    const [{ Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary, SkeletonJson }, skelResponse, atlasResponse] = await Promise.all([loadSpineModules(), fetch(skelURL + bust), fetch(atlasURL + bust)]);

    if (!skelResponse.ok) throw new Error(`Failed to load skeleton: ${skelResponse.status}`);
    if (!atlasResponse.ok) throw new Error(`Failed to load atlas: ${atlasResponse.status}`);

    const [skelData, atlasText] = await Promise.all([skelResponse.arrayBuffer(), atlasResponse.text()]);

    // Resolve texture page filenames relative to the atlas URL.
    const atlasBaseDir = atlasURL.slice(0, atlasURL.lastIndexOf("/") + 1);
    const pageInfo = parseAtlasPages(atlasText);

    const textureCache = new Map<string, PIXI.BaseTexture>();
    await Promise.all(
        Array.from(pageInfo, async ([pageName, { declaredW, declaredH }]) => {
            const img = await loadImage(`${atlasBaseDir}${encodeURIComponent(pageName)}${bust}`, { crossOrigin: true });
            textureCache.set(pageName, buildPageTexture(img, declaredW, declaredH));
        }),
    );

    const atlas = new TextureAtlas(atlasText, (pageName, callback) => {
        const tex = textureCache.get(pageName);
        if (tex) callback(tex);
    });

    // A `.skel` normally holds Spine binary data, but some dynchar illustrations
    // ship the skeleton as Spine JSON text under the same extension (e.g.
    // Eyjafjalla "A Picnic Before A Long Trip"'s idle). Sniff the first
    // non-whitespace byte: `{` => try JSON (a binary skel's leading hash byte
    // could coincidentally be `{`, so fall back to binary if the parse fails).
    const skelBytes = new Uint8Array(skelData);
    let firstByte = 0;
    for (let i = 0; i < skelBytes.length; i++) {
        const b = skelBytes[i];
        if (b === 0x20 || b === 0x09 || b === 0x0a || b === 0x0d || b === 0xef || b === 0xbb || b === 0xbf) continue; // ws / UTF-8 BOM
        firstByte = b;
        break;
    }
    const loader = new AtlasAttachmentLoader(atlas);
    let skeletonData: ReturnType<InstanceType<typeof SkeletonBinary>["readSkeletonData"]> | null = null;
    if (firstByte === 0x7b /* '{' */) {
        try {
            skeletonData = new SkeletonJson(loader).readSkeletonData(JSON.parse(new TextDecoder().decode(skelBytes)));
        } catch {
            skeletonData = null; // not actually JSON — fall through to binary
        }
    }
    if (!skeletonData) {
        skeletonData = new SkeletonBinary(loader).readSkeletonData(skelBytes);
    }

    // runtime-4.1 SkeletonData types diverge slightly from base Spine types,
    // but the runtime contract is compatible.
    const spine = new Spine(skeletonData as unknown as ConstructorParameters<typeof Spine>[0]);
    // Repair the runtime-3.8 PathConstraint rotation bug (see pathConstraintFix)
    // so Chain-mode path-constrained meshes (e.g. Zuo Le's legs) don't smear.
    patchSpine38PathConstraint(spine);
    // Dynamic-illustration L2D only: their animations are fully baked, so redundant
    // IK constraints that pixi-spine mis-solves (Archetto's arms) must yield to the
    // authoritative FK. Gated to DynIllust so battle/dorm chibis (IK-driven) are
    // untouched. (see bakedIkFix)
    if (skelPath.includes("DynIllust")) patchBakedIkRedundancy(spine);
    return spine;
}

/**
 * Resolve which skin to use given a requested name, falling back to "default"
 * and finally to the first available skin. Centralized so callers don't
 * duplicate the lookup logic.
 */
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

export interface IAnimationBounds {
    x: number;
    y: number;
    width: number;
    height: number;
}

const BOUNDS_SAMPLES = 40;

export function measureAnimationBounds(spine: Spine, animationName: string, samples = BOUNDS_SAMPLES): IAnimationBounds | null {
    const animation = spine.spineData.findAnimation(animationName);
    if (!animation) return null;

    const prevTimeScale = spine.state.timeScale;
    spine.state.timeScale = 1;
    spine.state.clearTracks();
    spine.skeleton.setToSetupPose();
    spine.state.setAnimation(0, animationName, true);

    const step = animation.duration > 0 ? animation.duration / samples : 0;
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;

    for (let i = 0; i <= samples; i++) {
        spine.update(i === 0 ? 0 : step);
        const b = spine.getLocalBounds();
        if (b.width > 0 && b.height > 0) {
            minX = Math.min(minX, b.x);
            minY = Math.min(minY, b.y);
            maxX = Math.max(maxX, b.x + b.width);
            maxY = Math.max(maxY, b.y + b.height);
        }
        if (step === 0) break;
    }

    spine.state.timeScale = prevTimeScale;

    if (maxX - minX <= 0 || maxY - minY <= 0) return null;
    return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

export function chibiMaxScale(canvasWidth: number, canvasHeight: number): number {
    return Math.min(canvasWidth / EXPORT_WIDTH, canvasHeight / EXPORT_HEIGHT) * CHIBI_SCALE;
}

export type SpineFitMode = "contain" | "cover" | "height";
export type SpineAlign = "top" | "center" | "bottom";
export interface ISpineFit {
    /**
     * `contain` = whole illustration visible (letterboxed); `cover` = fill the box,
     * cropping overflow; `height` = scale to the canvas HEIGHT only (the box's on-screen
     * height always equals the canvas height; excess width shows more scene, narrow width
     * crops the sides). `height` makes the subject fill a constant fraction of the frame
     * height regardless of the container's aspect ratio — the dynchar viewer uses it so the
     * character reads the same size on a tall mobile card, a wide desktop card, and a
     * fullscreen dialog alike (`cover` on a square box would inflate the subject with aspect).
     */
    mode: SpineFitMode;
    /** Vertical anchor of the illustration within the box. */
    align: SpineAlign;
}

export const DEFAULT_SPINE_FIT: ISpineFit = { mode: "contain", align: "center" };

/**
 * Position and scale a dynamic-illustration spine to fill a box, matching how
 * the static `<img>` underneath is framed (its `object-fit`/`object-position`).
 * When the two match, the animation lines up with the static art instead of
 * floating over it. Falls back to the chibi feet-at-bottom anchor when bounds
 * couldn't be measured.
 */
export function layoutSpine(target: PIXI.Container, canvasWidth: number, canvasHeight: number, bounds: IAnimationBounds | null, fit: ISpineFit = DEFAULT_SPINE_FIT): void {
    if (!bounds) {
        target.scale.set(chibiMaxScale(canvasWidth, canvasHeight));
        target.x = canvasWidth * CHIBI_OFFSET_X;
        target.y = canvasHeight * CHIBI_OFFSET_Y;
        return;
    }

    const scale = fit.mode === "cover" ? Math.max(canvasWidth / bounds.width, canvasHeight / bounds.height) : fit.mode === "height" ? canvasHeight / bounds.height : Math.min(canvasWidth / bounds.width, canvasHeight / bounds.height) * DYNAMIC_FIT_MARGIN;
    target.scale.set(scale);
    target.x = canvasWidth / 2 - (bounds.x + bounds.width / 2) * scale;
    switch (fit.align) {
        case "top":
            target.y = -bounds.y * scale;
            break;
        case "bottom":
            target.y = canvasHeight - (bounds.y + bounds.height) * scale;
            break;
        default:
            target.y = canvasHeight / 2 - (bounds.y + bounds.height / 2) * scale;
    }
}

export interface IExportLayout {
    width: number;
    height: number;
    chibiScale: number;
}

export function computeExportLayout(bounds: IAnimationBounds | null, scaleSetting: number): IExportLayout {
    const baseWidth = Math.round(EXPORT_WIDTH * scaleSetting);
    const baseHeight = Math.round(EXPORT_HEIGHT * scaleSetting);
    let chibiScale = chibiMaxScale(baseWidth, baseHeight);

    let width = baseWidth;
    let height = baseHeight;
    if (bounds) {
        width = Math.max(baseWidth, Math.ceil(bounds.width * chibiScale) + EXPORT_PADDING * 2);
        height = Math.max(baseHeight, Math.ceil(bounds.height * chibiScale) + EXPORT_PADDING * 2);

        const overshoot = Math.max(width, height) / MAX_EXPORT_DIM;
        if (overshoot > 1) {
            chibiScale /= overshoot;
            width = Math.max(baseWidth, Math.ceil(bounds.width * chibiScale) + EXPORT_PADDING * 2);
            height = Math.max(baseHeight, Math.ceil(bounds.height * chibiScale) + EXPORT_PADDING * 2);
        }
    }
    // H.264 encoders require even dimensions.
    width += width % 2;
    height += height % 2;

    return { width, height, chibiScale };
}

/** GIF frame delays are stored in centiseconds, so round to 10ms steps; browsers treat delays under 20ms as unreasonably fast and clamp them. */
export function gifFrameDelayMs(fps: number): number {
    return Math.max(20, Math.round(1000 / fps / 10) * 10);
}

export function effectiveGifFps(fps: number): number {
    return 1000 / gifFrameDelayMs(fps);
}

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
    return result;
}
