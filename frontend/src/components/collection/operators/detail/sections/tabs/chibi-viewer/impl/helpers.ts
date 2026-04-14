import * as PIXI from "pixi.js";
import type { ChibiCharacter, SpineFiles } from "~/types/api/impl/chibi";
import type { ViewType } from "./constants";

export function encodeAssetPath(path: string): string {
    if (!path) return "";
    return path
        .split("/")
        .map((segment) => {
            if (segment.includes("%")) return segment;
            return encodeURIComponent(segment);
        })
        .join("/");
}

export async function loadSpineWithEncodedUrls(skelUrl: string, atlasUrl: string, _pngUrl: string): Promise<import("pixi-spine").Spine> {
    const { Spine, TextureAtlas } = await import("pixi-spine");
    const { AtlasAttachmentLoader, SkeletonBinary } = await import("@pixi-spine/runtime-3.8");

    const [skelResponse, atlasResponse] = await Promise.all([fetch(skelUrl), fetch(atlasUrl)]);

    if (!skelResponse.ok) throw new Error(`Failed to load skeleton: ${skelResponse.status}`);
    if (!atlasResponse.ok) throw new Error(`Failed to load atlas: ${atlasResponse.status}`);

    const [skelData, atlasText] = await Promise.all([skelResponse.arrayBuffer(), atlasResponse.text()]);

    // Derive the base directory from the atlas URL so we can resolve
    // texture page names relative to it (the atlas references filenames like "build_char_4087_ines.png")
    const atlasBaseDir = atlasUrl.substring(0, atlasUrl.lastIndexOf("/") + 1);

    // Parse atlas text to find all page names and their declared sizes
    // Atlas format: page name on its own line, followed by "size: W,H"
    const pageInfo = new Map<string, { declaredW: number; declaredH: number }>();
    const lines = atlasText.split("\n");
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i]?.trim();
        const nextLine = lines[i + 1]?.trim();
        if (line && nextLine?.startsWith("size:") && line.endsWith(".png")) {
            const sizeMatch = nextLine.match(/size:\s*(\d+)\s*,\s*(\d+)/);
            if (sizeMatch) {
                pageInfo.set(line, {
                    declaredW: Number.parseInt(sizeMatch[1] ?? "0", 10),
                    declaredH: Number.parseInt(sizeMatch[2] ?? "0", 10),
                });
            }
        }
    }

    // Pre-load all texture pages before creating the atlas.
    // If the actual PNG is smaller than the atlas-declared size,
    // draw it onto a canvas at the declared size so coordinates are valid.
    const textureCache = new Map<string, PIXI.BaseTexture>();

    await Promise.all(
        Array.from(pageInfo.entries()).map(async ([pageName, { declaredW, declaredH }]) => {
            const pageUrl = `${atlasBaseDir}${encodeURIComponent(pageName)}`;

            // Load image to get actual dimensions
            const img = await new Promise<HTMLImageElement>((resolve, reject) => {
                const el = new Image();
                el.crossOrigin = "anonymous";
                el.onload = () => resolve(el);
                el.onerror = reject;
                el.src = pageUrl;
            });

            let baseTexture: PIXI.BaseTexture;

            if (img.naturalWidth < declaredW || img.naturalHeight < declaredH) {
                // PNG is smaller than atlas expects — scale up to declared size.
                // The game stores downscaled textures but atlas coords assume full resolution.
                const canvas = document.createElement("canvas");
                canvas.width = declaredW;
                canvas.height = declaredH;
                const ctx = canvas.getContext("2d");
                if (ctx) {
                    ctx.imageSmoothingEnabled = true;
                    ctx.imageSmoothingQuality = "high";
                    ctx.drawImage(img, 0, 0, declaredW, declaredH);
                }
                baseTexture = PIXI.BaseTexture.from(canvas);
            } else {
                baseTexture = PIXI.BaseTexture.from(img);
            }

            textureCache.set(pageName, baseTexture);
        }),
    );

    // Now create the atlas with all textures pre-loaded (synchronous callback)
    const textureCallback = (pageName: string, callback: (tex: PIXI.BaseTexture) => void) => {
        const tex = textureCache.get(pageName);
        if (tex) {
            callback(tex);
        }
    };

    const atlas = new TextureAtlas(atlasText, textureCallback);

    const attachmentLoader = new AtlasAttachmentLoader(atlas);
    const binaryLoader = new SkeletonBinary(attachmentLoader);
    const skeletonData = binaryLoader.readSkeletonData(new Uint8Array(skelData));

    // biome-ignore lint/suspicious/noExplicitAny: runtime-4.1 types are slightly different from base Spine types
    return new Spine(skeletonData as any);
}

export function getChibiSkinData(chibi: ChibiCharacter, skinName: string, viewType: ViewType): SpineFiles | null {
    const nameLower = skinName.toLowerCase();
    let skin = chibi.skins.find((s) => s.name.toLowerCase() === nameLower);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name.toLowerCase() === "default");
    }
    if (!skin && chibi.skins.length > 0) {
        skin = chibi.skins[0];
    }

    if (!skin || !skin.hasSpineData) return null;

    const animationType = skin.animationTypes[viewType];
    if (animationType?.atlas && animationType?.skel && animationType?.png) {
        return animationType;
    }

    const fallbackOrder: ViewType[] = ["front", "dorm", "back"];
    for (const fallbackType of fallbackOrder) {
        const fallback = skin.animationTypes[fallbackType];
        if (fallback?.atlas && fallback?.skel && fallback?.png) {
            return fallback;
        }
    }

    return null;
}

export function getAvailableViewTypes(chibi: ChibiCharacter, skinName: string): ViewType[] {
    const nameLower = skinName.toLowerCase();
    let skin = chibi.skins.find((s) => s.name.toLowerCase() === nameLower);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name.toLowerCase() === "default");
    }
    if (!skin && chibi.skins.length > 0) {
        skin = chibi.skins[0];
    }

    if (!skin) return [];

    const types: ViewType[] = [];
    if (skin.animationTypes.front?.skel) types.push("front");
    if (skin.animationTypes.back?.skel) types.push("back");
    if (skin.animationTypes.dorm?.skel) types.push("dorm");
    return types;
}
