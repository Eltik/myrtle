import * as PIXI from "pixi.js";
import type { Spine } from "pixi-spine";
import { env } from "#/env";
import type { IChibiCharacter, IChibiSkin, IChibiSpineFiles } from "#/lib/api/chibis";
import { loadImage } from "#/lib/utils";
import type { ViewType } from "./constants";

let spineModulesPromise: Promise<{
    Spine: typeof import("pixi-spine").Spine;
    TextureAtlas: typeof import("pixi-spine").TextureAtlas;
    AtlasAttachmentLoader: typeof import("@pixi-spine/runtime-3.8").AtlasAttachmentLoader;
    SkeletonBinary: typeof import("@pixi-spine/runtime-3.8").SkeletonBinary;
}> | null = null;

function loadSpineModules() {
    if (!spineModulesPromise) {
        spineModulesPromise = Promise.all([import("pixi-spine"), import("@pixi-spine/runtime-3.8")]).then(([spine, runtime]) => ({
            Spine: spine.Spine,
            TextureAtlas: spine.TextureAtlas,
            AtlasAttachmentLoader: runtime.AtlasAttachmentLoader,
            SkeletonBinary: runtime.SkeletonBinary,
        }));
    }
    return spineModulesPromise;
}

const VIEW_FALLBACK_ORDER: readonly ViewType[] = ["front", "dorm", "back"] as const;

// Matches an atlas page declaration: a line ending in ".png" followed by "size: W,H".
// Capturing groups: 1=page name, 2=width, 3=height.
const ATLAS_PAGE_RE = /^(.+\.png)\r?\n\s*size:\s*(\d+)\s*,\s*(\d+)/gm;

export function encodeAssetPath(path: string): string {
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

function chibiAssetURL(path: string): string {
    return `${env.VITE_BACKEND_URL}/api/assets${encodeAssetPath(path)}`;
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

export async function loadSpineWithEncodedURLs(skelPath: string, atlasPath: string): Promise<Spine> {
    const skelURL = chibiAssetURL(skelPath);
    const atlasURL = chibiAssetURL(atlasPath);

    const [{ Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary }, skelResponse, atlasResponse] = await Promise.all([loadSpineModules(), fetch(skelURL), fetch(atlasURL)]);

    if (!skelResponse.ok) throw new Error(`Failed to load skeleton: ${skelResponse.status}`);
    if (!atlasResponse.ok) throw new Error(`Failed to load atlas: ${atlasResponse.status}`);

    const [skelData, atlasText] = await Promise.all([skelResponse.arrayBuffer(), atlasResponse.text()]);

    // Resolve texture page filenames relative to the atlas URL.
    const atlasBaseDir = atlasURL.slice(0, atlasURL.lastIndexOf("/") + 1);
    const pageInfo = parseAtlasPages(atlasText);

    const textureCache = new Map<string, PIXI.BaseTexture>();
    await Promise.all(
        Array.from(pageInfo, async ([pageName, { declaredW, declaredH }]) => {
            const img = await loadImage(`${atlasBaseDir}${encodeURIComponent(pageName)}`, { crossOrigin: true });
            textureCache.set(pageName, buildPageTexture(img, declaredW, declaredH));
        }),
    );

    const atlas = new TextureAtlas(atlasText, (pageName, callback) => {
        const tex = textureCache.get(pageName);
        if (tex) callback(tex);
    });

    const skeletonData = new SkeletonBinary(new AtlasAttachmentLoader(atlas)).readSkeletonData(new Uint8Array(skelData));

    // runtime-4.1 SkeletonData types diverge slightly from base Spine types,
    // but the runtime contract is compatible.
    return new Spine(skeletonData as unknown as ConstructorParameters<typeof Spine>[0]);
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

function isCompleteAnimation(anim: IChibiSpineFiles | undefined | null): anim is IChibiSpineFiles {
    return !!anim?.atlas && !!anim?.skel && !!anim?.png;
}

export function getChibiSkinData(chibi: IChibiCharacter, skinName: string, viewType: ViewType): IChibiSpineFiles | null {
    const skin = resolveSkin(chibi, skinName);
    if (!skin) return null;

    const types = skin.animationTypes;
    if (!types.front?.skel && !types.back?.skel && !types.dorm?.skel) return null;

    if (isCompleteAnimation(types[viewType])) return types[viewType] as IChibiSpineFiles;

    for (const fallbackType of VIEW_FALLBACK_ORDER) {
        const fallback = types[fallbackType];
        if (isCompleteAnimation(fallback)) return fallback;
    }

    return null;
}

export function getAvailableViewTypes(chibi: IChibiCharacter | null, skinName: string | null): ViewType[] {
    if (!chibi || !skinName) return [];
    const skin = resolveSkin(chibi, skinName);
    if (!skin) return [];

    const types = skin.animationTypes;
    const result: ViewType[] = [];
    if (types.front?.skel) result.push("front");
    if (types.back?.skel) result.push("back");
    if (types.dorm?.skel) result.push("dorm");
    return result;
}
