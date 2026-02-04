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

// Custom texture loader that properly encodes URLs with # characters
export async function loadSpineWithEncodedUrls(skelUrl: string, atlasUrl: string, pngUrl: string): Promise<import("pixi-spine").Spine> {
    // Import required classes - Arknights uses Spine 3.8 format
    const { Spine, TextureAtlas } = await import("pixi-spine");
    const { AtlasAttachmentLoader, SkeletonBinary } = await import("@pixi-spine/runtime-3.8");

    // Load all files in parallel
    const [skelResponse, atlasResponse] = await Promise.all([fetch(skelUrl), fetch(atlasUrl)]);

    if (!skelResponse.ok) throw new Error(`Failed to load skeleton: ${skelResponse.status}`);
    if (!atlasResponse.ok) throw new Error(`Failed to load atlas: ${atlasResponse.status}`);

    const [skelData, atlasText] = await Promise.all([skelResponse.arrayBuffer(), atlasResponse.text()]);

    // Load the texture
    const texture = await PIXI.Assets.load(pngUrl);

    // Create a simple texture callback for the atlas
    const textureCallback = (_path: string, callback: (tex: PIXI.BaseTexture) => void) => {
        callback(texture.baseTexture);
    };

    // Parse the atlas with our texture
    const atlas = new TextureAtlas(atlasText, textureCallback);

    // Create the skeleton
    const attachmentLoader = new AtlasAttachmentLoader(atlas);
    const binaryLoader = new SkeletonBinary(attachmentLoader);
    const skeletonData = binaryLoader.readSkeletonData(new Uint8Array(skelData));

    // biome-ignore lint/suspicious/noExplicitAny: runtime-4.1 types are slightly different from base Spine types
    return new Spine(skeletonData as any);
}

export function getChibiSkinData(chibi: ChibiCharacter, skinName: string, viewType: ViewType): SpineFiles | null {
    let skin = chibi.skins.find((s) => s.name === skinName);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name === "default");
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
    let skin = chibi.skins.find((s) => s.name === skinName);
    if (!skin) {
        skin = chibi.skins.find((s) => s.name === "default");
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
